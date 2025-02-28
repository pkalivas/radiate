use super::aggregate::GraphAggregate;
use crate::{
    Arity, Factory, NodeStore,
    collections::{Graph, GraphNode, NodeType},
};

impl<T: Clone + Default> Graph<T> {
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

    pub fn recurrent(
        input_size: usize,
        output_size: usize,
        values: impl Into<NodeStore<T>>,
    ) -> Graph<T> {
        let builder = NodeBuilder::new(values);

        let input = builder.input(input_size);
        let aggregate = builder.vertecies(input_size);
        let link = builder.vertecies(input_size);
        let output = builder.output(output_size);

        GraphAggregate::new()
            .one_to_one(&input, &aggregate)
            .one_to_one_self(&aggregate, &link)
            .all_to_all(&aggregate, &output)
            .build()
    }

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

    pub fn weighted_recurrent(
        input_size: usize,
        output_size: usize,
        values: impl Into<NodeStore<T>>,
    ) -> Graph<T> {
        let builder = NodeBuilder::new(values);

        let input = builder.input(input_size);
        let aggregate = builder.vertecies(input_size);
        let link = builder.vertecies(input_size);
        let output = builder.output(output_size);
        let weights = builder.edge(input_size * input_size);

        GraphAggregate::new()
            .one_to_one(&input, &aggregate)
            .one_to_one_self(&aggregate, &link)
            .one_to_many(&link, &weights)
            .many_to_one(&weights, &output)
            .build()
    }
}

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

    pub fn vertecies(&self, size: usize) -> Vec<GraphNode<T>> {
        (0..size)
            .map(|idx| {
                self.store
                    .new_instance((idx, NodeType::Vertex, |arity| match arity {
                        Arity::Any => true,
                        _ => false,
                    }))
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
    use crate::{Node, Op};

    use super::*;

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

        assert_eq!(graph.len(), 12);

        for node in graph.iter() {
            if node.node_type() == NodeType::Input {
                assert_eq!(node.arity(), Arity::Zero);
                assert_eq!(node.incoming().iter().count(), 0);
                assert_eq!(node.outgoing().iter().count(), 1);
            } else if node.node_type() == NodeType::Vertex {
                assert_eq!(node.arity(), Arity::Any);
                assert_eq!(node.is_recurrent(), true);
                assert_eq!(node.value(), &Op::sigmoid());
            } else if node.node_type() == NodeType::Output {
                assert_eq!(node.arity(), Arity::Any);
                assert_eq!(node.incoming().iter().count(), 3);
                assert_eq!(node.outgoing().iter().count(), 0);
                assert_eq!(node.value(), &Op::sigmoid());
            }
        }
    }
}

// pub fn gru(
//     mut self,
//     input_size: usize,
//     output_size: usize,
//     memory_size: usize,
//     output: Op<f32>,
// ) -> GraphBuilder<f32> {
//     self.with_values(NodeType::Input, (0..input_size).map(Op::var).collect());
//     self.with_values(NodeType::Output, vec![output]);

//     let input = self.input(input_size);
//     let output = self.output(output_size);

//     let output_weights = self.edge(memory_size * output_size);

//     let reset_gate = self.aggregates(memory_size);
//     let update_gate = self.aggregates(memory_size);
//     let candidate_gate = self.aggregates(memory_size);

//     let input_to_reset_weights = self.edge(input_size * memory_size);
//     let input_to_update_weights = self.edge(input_size * memory_size);
//     let input_to_candidate_weights = self.edge(input_size * memory_size);

//     let hidden_to_reset_weights = self.edge(memory_size * memory_size);
//     let hidden_to_update_weights = self.edge(memory_size * memory_size);
//     let hidden_to_candidate_weights = self.edge(memory_size * memory_size);

//     let hidden_reset_gate = self.aggregates(memory_size);
//     let update_candidate_mul_gate = self.aggregates(memory_size);
//     let invert_update_gate = self.aggregates(memory_size);
//     let hidden_invert_mul_gate = self.aggregates(memory_size);
//     let candidate_hidden_add_gate = self.aggregates(memory_size);

//     let graph = GraphAggregate::new()
//         .one_to_many(&input, &input_to_reset_weights)
//         .one_to_many(&input, &input_to_update_weights)
//         .one_to_many(&input, &input_to_candidate_weights)
//         .one_to_many(&candidate_hidden_add_gate, &hidden_to_reset_weights)
//         .one_to_many(&candidate_hidden_add_gate, &hidden_to_update_weights)
//         .many_to_one(&input_to_reset_weights, &reset_gate)
//         .many_to_one(&hidden_to_reset_weights, &reset_gate)
//         .many_to_one(&input_to_update_weights, &update_gate)
//         .many_to_one(&hidden_to_update_weights, &update_gate)
//         .one_to_one(&reset_gate, &hidden_reset_gate)
//         .one_to_one(&candidate_hidden_add_gate, &hidden_reset_gate)
//         .one_to_many(&hidden_reset_gate, &hidden_to_candidate_weights)
//         .many_to_one(&input_to_candidate_weights, &candidate_gate)
//         .many_to_one(&hidden_to_candidate_weights, &candidate_gate)
//         .one_to_one(&update_gate, &update_candidate_mul_gate)
//         .one_to_one(&candidate_gate, &update_candidate_mul_gate)
//         .one_to_one(&update_gate, &invert_update_gate)
//         .one_to_one(&candidate_hidden_add_gate, &hidden_invert_mul_gate)
//         .one_to_one(&invert_update_gate, &hidden_invert_mul_gate)
//         .one_to_one(&hidden_invert_mul_gate, &candidate_hidden_add_gate)
//         .one_to_one(&update_candidate_mul_gate, &candidate_hidden_add_gate)
//         .one_to_many(&candidate_hidden_add_gate, &output_weights)
//         .many_to_one(&output_weights, &output)
//         .build();

//     self.node_cache = Some(graph.into_iter().collect());
//     self
// }

// pub fn lstm(mut self, input_size: usize, output_size: usize, output: Op<f32>) -> GraphBuilder<f32> {
//     self.with_values(NodeType::Input, (0..input_size).map(Op::var).collect());
//     self.with_values(NodeType::Output, vec![output]);

//     let input = self.input(input_size);
//     let output = self.output(output_size);

//     let cell_state = self.aggregates(1);
//     let hidden_state = self.aggregates(1);

//     let forget_gate = self.aggregates(1);
//     let input_gate = self.aggregates(1);
//     let output_gate = self.aggregates(1);
//     let candidate = self.aggregates(1);

//     let input_to_forget_weights = self.edge(input_size);
//     let input_to_input_weights = self.edge(input_size);
//     let input_to_output_weights = self.edge(input_size);
//     let input_to_candidate_weights = self.edge(input_size);

//     let hidden_to_forget_weights = self.edge(1);
//     let hidden_to_input_weights = self.edge(1);
//     let hidden_to_output_weights = self.edge(1);
//     let hidden_to_candidate_weights = self.edge(1);

//     let final_weights = self.edge(output_size);

//     let graph = GraphAggregate::new()
//         .one_to_one(&input, &input_to_forget_weights)
//         .one_to_one(&input, &input_to_input_weights)
//         .one_to_one(&input, &input_to_output_weights)
//         .one_to_one(&input, &input_to_candidate_weights)
//         .one_to_one(&hidden_state, &hidden_to_forget_weights)
//         .one_to_one(&hidden_state, &hidden_to_input_weights)
//         .one_to_one(&hidden_state, &hidden_to_output_weights)
//         .one_to_one(&hidden_state, &hidden_to_candidate_weights)
//         .many_to_one(&input_to_forget_weights, &forget_gate)
//         .many_to_one(&hidden_to_forget_weights, &forget_gate)
//         .many_to_one(&input_to_input_weights, &input_gate)
//         .many_to_one(&hidden_to_input_weights, &input_gate)
//         .many_to_one(&input_to_output_weights, &output_gate)
//         .many_to_one(&hidden_to_output_weights, &output_gate)
//         .many_to_one(&input_to_candidate_weights, &candidate)
//         .many_to_one(&hidden_to_candidate_weights, &candidate)
//         .one_to_one(&forget_gate, &cell_state)
//         .one_to_one(&input_gate, &candidate)
//         .one_to_one(&candidate, &cell_state)
//         .one_to_one(&cell_state, &hidden_state)
//         .one_to_one(&output_gate, &hidden_state)
//         .one_to_many(&hidden_state, &final_weights)
//         .one_to_one(&final_weights, &output)
//         .build();

//     self.node_cache = Some(graph.into_iter().collect());
//     self
// }
