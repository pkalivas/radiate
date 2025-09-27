use super::aggregate::GraphAggregate;
use crate::{
    Arity, Factory, NodeStore,
    collections::{Graph, GraphNode, NodeType},
};

impl<T: Clone + Default> Graph<T> {
    /// Creates a directed graph with the given input, output sizes and values.
    /// The values are used to initialize the nodes in the graph with the given values.
    ///
    /// # Example
    /// ```
    /// use radiate_gp::*;
    ///
    /// let values = vec![
    ///     (NodeType::Input, vec![Op::var(0), Op::var(1), Op::var(2)]),
    ///     (NodeType::Output, vec![Op::sigmoid()]),
    /// ];
    ///
    /// let graph = Graph::directed(3, 3, values);
    ///
    /// assert_eq!(graph.len(), 6);
    /// ```
    ///
    /// The graph will have 6 nodes, 3 input nodes and 3 output nodes where each input node is
    /// connected to each output node. Such as:
    /// ``` text
    /// [0, 1, 2] -> [3, 4, 5]
    /// ```
    ///
    /// # Arguments
    /// * `input_size` - The number of input nodes.
    /// * `output_size` - The number of output nodes.
    /// * `values` - The values to initialize the nodes with.
    ///
    /// # Returns
    /// A new directed graph.
    pub fn directed(
        input_size: usize,
        output_size: usize,
        values: impl Into<NodeStore<T>>,
    ) -> Graph<T> {
        let builder = NodeBuilder::new(values);

        let input_nodes = builder.input(input_size);
        let output_nodes = builder.output(output_size);

        GraphAggregate::new()
            .all_to_all(&input_nodes, &output_nodes)
            .build()
    }

    /// Creates a recurrent graph with the given input and output sizes.
    /// The values are used to initialize the nodes in the graph with the given values.
    /// The graph will have a recurrent connection from each hidden vertex to itself.
    /// The graph will have a one-to-one connection from each input node to each hidden vertex.
    /// The graph will have an all-to-all connection from each hidden vertex to each output node.
    ///
    /// # Example
    /// ```
    /// use radiate_gp::*;
    ///
    /// let values = vec![
    ///   (NodeType::Input, vec![Op::var(0), Op::var(1), Op::var(2)]),
    ///   (NodeType::Vertex, vec![Op::linear()]),
    ///   (NodeType::Output, vec![Op::sigmoid()]),
    /// ];
    ///
    /// let graph = Graph::recurrent(3, 3, values);
    ///
    /// assert_eq!(graph.len(), 9);
    /// ```
    ///
    /// The graph will have 9 nodes, 3 input nodes, 3 hidden nodes with recurrent connections to themselves,
    /// and 3 output nodes. Such as:
    /// ``` text
    /// [0, 1, 2] -> [3, 4, 5]
    ///     [3, 4, 5] -> [6, 7, 8]
    ///         [6, 7, 8] -> [3, 4, 5]
    /// [3, 4, 5] -> [9, 10, 11]
    /// ```
    ///
    /// # Arguments
    /// * `input_size` - The number of input nodes.
    /// * `output_size` - The number of output nodes.
    /// * `values` - The values to initialize the nodes with.
    ///
    /// # Returns
    /// A new recurrent graph.
    pub fn recurrent(
        input_size: usize,
        output_size: usize,
        values: impl Into<NodeStore<T>>,
    ) -> Graph<T> {
        let builder = NodeBuilder::new(values);

        let input = builder.input(input_size);
        let aggregate = builder.vertices(input_size);
        let output = builder.output(output_size);

        GraphAggregate::new()
            .one_to_one(&input, &aggregate)
            .cycle(&aggregate)
            .all_to_all(&aggregate, &output)
            .build()
    }

    /// Creates a weighted directed graph with the given input and output sizes.
    ///
    /// This will result in the same graph as `Graph::directed` but with an additional edge
    /// connecting each input node to each output node.
    ///
    /// # Arguments
    /// * `input_size` - The number of input nodes.
    /// * `output_size` - The number of output nodes.
    ///
    /// # Returns
    /// A new weighted directed graph.
    pub fn weighted_directed(
        input_size: usize,
        output_size: usize,
        values: impl Into<NodeStore<T>>,
    ) -> Graph<T> {
        let builder = NodeBuilder::new(values);

        let input = builder.input(input_size);
        let output = builder.output(output_size);
        let weights = builder.edge(input_size * output_size);

        GraphAggregate::new()
            .one_to_many(&input, &weights)
            .many_to_one(&weights, &output)
            .build()
    }

    /// Creates a weighted recurrent graph with the given input and output sizes.
    /// This will result in the same graph as `Graph::recurrent` but with an additional edge
    /// connecting each hidden vertex to each output node.
    ///
    /// # Arguments
    /// * `input_size` - The number of input nodes.
    /// * `output_size` - The number of output nodes.
    ///
    /// # Returns
    /// A new weighted recurrent graph.
    pub fn weighted_recurrent(
        input_size: usize,
        output_size: usize,
        values: impl Into<NodeStore<T>>,
    ) -> Graph<T> {
        let builder = NodeBuilder::new(values);

        let input = builder.input(input_size);
        let aggregate = builder.vertices(input_size);
        let output = builder.output(output_size);
        let weights = builder.edge(input_size * output_size);

        GraphAggregate::new()
            .one_to_one(&input, &aggregate)
            .cycle(&aggregate)
            .one_to_many(&aggregate, &weights)
            .many_to_one(&weights, &output)
            .build()
    }

    /// Creates a Long Short-Term Memory (LSTM) graph with the given input and output sizes.
    /// The graph will have the following structure:
    /// - Input nodes connected to forget, input, candidate, and output gates.
    /// - Hidden state connected to forget, input, candidate, and output gates.
    /// - Forget gate connected to cell state.
    /// - Input gate connected to candidate and cell state.
    /// - Candidate connected to cell state.
    /// - Cell state connected to hidden state.
    /// - Output gate connected to hidden state.
    /// - Hidden state connected to output nodes.
    ///
    /// # Arguments
    /// * `input_size` - The number of input nodes.
    /// * `output_size` - The number of output nodes.
    /// * `store` - The node store.
    ///
    /// # Returns
    /// A new LSTM graph.
    pub fn lstm(input_size: usize, output_size: usize, store: impl Into<NodeStore<T>>) -> Graph<T> {
        let builder = NodeBuilder::new(store);

        let input = builder.input(input_size);
        let output = builder.output(output_size);

        let cell_state = builder.vertices(1);
        let hidden_state = builder.vertices(1);

        let forget_gate = builder.vertices(1);
        let input_gate = builder.vertices(1);
        let output_gate = builder.vertices(1);
        let candidate = builder.vertices(1);

        GraphAggregate::new()
            .all_to_all(&input, &forget_gate)
            .all_to_all(&input, &input_gate)
            .all_to_all(&input, &output_gate)
            .all_to_all(&input, &candidate)
            .one_to_one(&hidden_state, &forget_gate)
            .one_to_one(&hidden_state, &input_gate)
            .one_to_one(&hidden_state, &output_gate)
            .one_to_one(&hidden_state, &candidate)
            .one_to_one(&forget_gate, &cell_state)
            .one_to_one(&input_gate, &candidate)
            .one_to_one(&candidate, &cell_state)
            .one_to_one(&cell_state, &hidden_state)
            .one_to_one(&output_gate, &hidden_state)
            .all_to_all(&hidden_state, &output)
            .build()
    }

    /// Creates a Gated Recurrent Unit (GRU) graph with the given input and output sizes.
    /// The graph will have the following structure:
    /// - Input nodes connected to reset, update, and candidate gates.
    /// - Hidden state connected to reset, update, and candidate gates.
    /// - Reset gate connected to hidden state.
    /// - Update gate connected to blend and gate flip.
    /// - Candidate connected to blend.
    /// - Blend connected to hidden state.
    /// - Gate flip connected to hidden state.
    /// - Hidden state connected to output nodes.
    ///
    /// # Arguments
    /// * `input_size` - The number of input nodes.
    /// * `output_size` - The number of output nodes.
    /// * `store` - The node store.
    ///
    /// # Returns
    /// A new GRU graph.
    pub fn gru(input_size: usize, output_size: usize, values: impl Into<NodeStore<T>>) -> Graph<T> {
        let builder = NodeBuilder::new(values);

        let input = builder.input(input_size);
        let output = builder.output(output_size);

        let hidden = builder.vertices(1);

        let update = builder.vertices(1);
        let reset = builder.vertices(1);
        let candidate = builder.vertices(1);

        let blend = builder.vertices(1);
        let gate_flip = builder.vertices(1);

        GraphAggregate::new()
            .many_to_one(&input, &reset)
            .many_to_one(&input, &update)
            .many_to_one(&input, &candidate)
            .one_to_one(&hidden, &reset)
            .one_to_one(&hidden, &update)
            .one_to_one(&hidden, &candidate)
            .one_to_one(&update, &blend)
            .one_to_one(&candidate, &blend)
            .one_to_one(&reset, &hidden)
            .one_to_one(&update, &gate_flip)
            .one_to_one(&hidden, &gate_flip)
            .one_to_one(&gate_flip, &hidden)
            .one_to_one(&blend, &hidden)
            .one_to_many(&hidden, &output)
            .build()
    }

    /// Creates a 2D mesh graph with bidirectional connections between neighboring nodes.
    /// The graph will have the following structure:
    /// - Input nodes connected to the first row of mesh nodes.
    /// - Each mesh node connected to its neighbors (up, down, left, right).
    /// - Last row of mesh nodes connected to output nodes.
    ///
    /// # Arguments
    /// * `width` - The number of nodes in the horizontal dimension.
    /// * `height` - The number of nodes in the vertical dimension.
    /// * `values` - The values to initialize the nodes with.
    ///
    /// # Returns
    /// A new 2D mesh graph.
    pub fn mesh(
        input_size: usize,
        output_size: usize,
        width: usize,
        height: usize,
        values: impl Into<NodeStore<T>>,
    ) -> Graph<T> {
        let builder = NodeBuilder::new(values);

        let inputs = builder.input(input_size);
        let outputs = builder.output(output_size);
        let nodes = (0..width * height)
            .map(|_| builder.vertices(1))
            .collect::<Vec<Vec<GraphNode<T>>>>();

        let mut aggregate = GraphAggregate::new();

        for y in 0..height {
            for x in 0..width {
                let index = y * width + x;
                let current = &nodes[index];

                if x + 1 < width {
                    let right = &nodes[y * width + (x + 1)];
                    aggregate = aggregate.one_to_one(current, right);
                }

                if y + 1 < height {
                    let down = &nodes[(y + 1) * width + x];
                    aggregate = aggregate.one_to_one(current, down);
                }
            }
        }

        aggregate
            .many_to_one(&inputs, &nodes[0])
            .one_to_many(&nodes[nodes.len() - 1], &outputs)
            .build()
    }
}

/// A simple builder struct for constructing nodes of a certain type. This is pretty much just a
/// quality of life struct that removes boilerplate code when creating collections of nodes.
struct NodeBuilder<T> {
    store: NodeStore<T>,
}

impl<T: Clone + Default> NodeBuilder<T> {
    pub fn new(store: impl Into<NodeStore<T>>) -> Self {
        NodeBuilder {
            store: store.into(),
        }
    }

    pub fn input(&self, size: usize) -> Vec<GraphNode<T>> {
        self.new_nodes(NodeType::Input, size, Arity::Zero)
    }

    pub fn output(&self, size: usize) -> Vec<GraphNode<T>> {
        self.new_nodes(NodeType::Output, size, Arity::Any)
    }

    pub fn edge(&self, size: usize) -> Vec<GraphNode<T>> {
        self.new_nodes(NodeType::Edge, size, Arity::Exact(1))
    }

    pub fn vertices(&self, size: usize) -> Vec<GraphNode<T>> {
        (0..size)
            .map(|idx| {
                self.store
                    .new_instance((idx, NodeType::Vertex, |arity| matches!(arity, Arity::Any)))
            })
            .collect()
    }

    fn new_nodes(
        &self,
        node_type: NodeType,
        size: usize,
        fallback_arity: Arity,
    ) -> Vec<GraphNode<T>> {
        if self.store.contains_type(node_type) {
            (0..size)
                .map(|idx| self.store.new_instance((idx, node_type)))
                .collect()
        } else {
            (0..size)
                .map(|idx| {
                    self.store
                        .new_instance((idx, node_type, |arity| arity == fallback_arity))
                })
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Node, Op, node_store};
    use radiate_core::Valid;

    #[test]
    fn test_graph_builder() {
        let graph = Graph::directed(3, 3, Op::sigmoid());

        assert_eq!(graph.len(), 6);

        for node in graph.iter() {
            if node.node_type() == NodeType::Input {
                assert_eq!(node.arity(), Arity::Zero);
                assert_eq!(node.incoming().iter().count(), 0);
                assert_eq!(node.outgoing().iter().count(), 3);
            } else if node.node_type() == NodeType::Output {
                assert_eq!(node.arity(), Arity::Any);
                assert_eq!(node.incoming().iter().count(), 3);
                assert_eq!(node.outgoing().iter().count(), 0);
                assert_eq!(node.value(), &Op::sigmoid());
            }
        }
    }

    #[test]
    fn test_graph_builder_recurrent() {
        let graph = Graph::recurrent(3, 3, Op::sigmoid());

        assert_eq!(graph.len(), 9);

        for node in graph.iter() {
            if node.node_type() == NodeType::Input {
                assert_eq!(node.arity(), Arity::Zero);
                assert_eq!(node.incoming().iter().count(), 0);
                assert_eq!(node.outgoing().iter().count(), 1);
            } else if node.node_type() == NodeType::Vertex {
                assert_eq!(node.arity(), Arity::Any);
                assert!(node.is_recurrent());
                assert_eq!(node.value(), &Op::sigmoid());
            } else if node.node_type() == NodeType::Output {
                assert_eq!(node.arity(), Arity::Any);
                assert_eq!(node.incoming().iter().count(), 3);
                assert_eq!(node.outgoing().iter().count(), 0);
                assert_eq!(node.value(), &Op::sigmoid());
            }
        }
    }

    #[test]
    fn test_graph_builder_with_no_any() {
        let graph = Graph::directed(3, 3, Op::add());

        assert_eq!(graph.len(), 6);
        assert!(!graph.is_valid());
    }

    #[test]
    fn test_graph_builder_weighted() {
        let store = vec![
            (NodeType::Input, vec![Op::var(0), Op::var(1), Op::var(2)]),
            (NodeType::Output, vec![Op::sigmoid()]),
            (NodeType::Edge, vec![Op::weight_with(1.0)]),
        ];

        let graph = Graph::weighted_directed(3, 3, store);

        assert_eq!(graph.len(), 15);
        assert!(graph.is_valid());

        for node in graph.iter() {
            if node.node_type() == NodeType::Input {
                assert_eq!(node.arity(), Arity::Zero);
                assert_eq!(node.incoming().iter().count(), 0);
                assert_eq!(node.outgoing().iter().count(), 3);
            } else if node.node_type() == NodeType::Edge {
                assert_eq!(node.arity(), Arity::Exact(1));
                assert_eq!(node.incoming().iter().count(), 1);
                assert_eq!(node.outgoing().iter().count(), 1);
                assert_eq!(node.value(), &Op::weight_with(1.0));
            } else if node.node_type() == NodeType::Output {
                assert_eq!(node.arity(), Arity::Any);
                assert_eq!(node.incoming().iter().count(), 3);
                assert_eq!(node.outgoing().iter().count(), 0);
                assert_eq!(node.value(), &Op::sigmoid());
            }
        }
    }

    #[test]
    fn test_graph_builder_weighted_recurrent() {
        let store = node_store![
            Input => vec![Op::var(0), Op::var(1), Op::var(2)],
            Output => vec![Op::sigmoid()],
            Edge => vec![Op::weight_with(1.0)]
        ];

        let graph = Graph::weighted_recurrent(3, 3, store);

        assert_eq!(graph.len(), 18);
        assert!(graph.is_valid());

        for node in graph.iter() {
            if node.node_type() == NodeType::Input {
                assert_eq!(node.arity(), Arity::Zero);
                assert_eq!(node.incoming().iter().count(), 0);
                assert_eq!(node.outgoing().iter().count(), 1);
            } else if node.node_type() == NodeType::Edge {
                assert_eq!(node.arity(), Arity::Exact(1));
                assert_eq!(node.incoming().iter().count(), 1);
                assert_eq!(node.outgoing().iter().count(), 1);
                assert_eq!(node.value(), &Op::weight_with(1.0));
            } else if node.node_type() == NodeType::Output {
                assert_eq!(node.arity(), Arity::Any);
                assert_eq!(node.incoming().iter().count(), 3);
                assert_eq!(node.outgoing().iter().count(), 0);
                assert_eq!(node.value(), &Op::sigmoid());
            } else if node.node_type() == NodeType::Vertex {
                assert_eq!(node.arity(), Arity::Any);
                assert!(node.is_recurrent());
                assert_eq!(node.value(), &Op::sigmoid());
            }
        }
    }

    #[test]
    fn test_graph_builder_lstm() {
        let store = node_store![
            Input => vec![Op::var(0)],
            Output => vec![Op::sigmoid()],
            Vertex => vec![Op::sigmoid(), Op::tanh(), Op::mul(), Op::add()],
            Edge => vec![Op::weight_with(1.0)]
        ];

        let graph = Graph::lstm(1, 1, store);
        assert_eq!(graph.len(), 8);
        assert!(graph.is_valid());

        for node in graph.iter() {
            if node.node_type() == NodeType::Input {
                assert_eq!(node.arity(), Arity::Zero);
                assert_eq!(node.incoming().iter().count(), 0);
                assert_eq!(node.outgoing().iter().count(), 4);
            } else if node.node_type() == NodeType::Output {
                assert_eq!(node.arity(), Arity::Any);
                assert_eq!(node.incoming().iter().count(), 1);
                assert_eq!(node.outgoing().iter().count(), 0);
                assert_eq!(node.value(), &Op::sigmoid());
            } else if node.node_type() == NodeType::Vertex {
                assert_eq!(node.arity(), Arity::Any);
                assert!(
                    vec![Op::sigmoid(), Op::tanh(), Op::mul(), Op::add()].contains(&node.value())
                );
            } else if node.node_type() == NodeType::Edge {
                assert_eq!(node.arity(), Arity::Exact(1));
                assert_eq!(node.incoming().iter().count(), 1);
                assert_eq!(node.outgoing().iter().count(), 1);
                assert_eq!(node.value(), &Op::weight_with(1.0));
            }
        }
    }

    #[test]
    fn test_graph_builder_gru() {
        let store = node_store![
            Input => vec![Op::var(0)],
            Output => vec![Op::sigmoid()],
            Vertex => vec![Op::sigmoid(), Op::tanh(), Op::mul(), Op::add()],
            Edge => vec![Op::weight_with(1.0)]
        ];

        let graph = Graph::gru(1, 1, store);

        assert_eq!(graph.len(), 8);
        assert!(graph.is_valid());

        for node in graph.iter() {
            if node.node_type() == NodeType::Input {
                assert_eq!(node.arity(), Arity::Zero);
                assert_eq!(node.incoming().iter().count(), 0);
                assert_eq!(node.outgoing().iter().count(), 3);
            } else if node.node_type() == NodeType::Output {
                assert_eq!(node.arity(), Arity::Any);
                assert_eq!(node.incoming().iter().count(), 1);
                assert_eq!(node.outgoing().iter().count(), 0);
                assert_eq!(node.value(), &Op::sigmoid());
            } else if node.node_type() == NodeType::Vertex {
                assert_eq!(node.arity(), Arity::Any);
                assert!(
                    vec![Op::sigmoid(), Op::tanh(), Op::mul(), Op::add()].contains(&node.value())
                );
            } else if node.node_type() == NodeType::Edge {
                assert_eq!(node.arity(), Arity::Exact(1));
                assert_eq!(node.incoming().iter().count(), 1);
                assert_eq!(node.outgoing().iter().count(), 1);
                assert_eq!(node.value(), &Op::weight_with(1.0));
            }
        }
    }
}
