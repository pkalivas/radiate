use super::node::GraphNodeId;
use crate::{
    collections::{Graph, GraphNode, NodeType},
    node::Node,
};
use std::collections::BTreeMap;

/// Building a [Graph<T>] can be a very complex task. Everything in this file exists
/// to simplify the process by allowing the user to do so in a declarative way.
///
/// The [ConnectTypes] are a set of available ways we can connect different [GraphNode]'s together.
///
/// # Assumptions
/// * The first collection is the 'source' collection and the second collection is the 'target' collection.
/// * The target collection's [GraphNode]'s `Arity` is compatible with the [ConnectTypes].
enum ConnectTypes {
    /// Connects each `GraphNode` in the first collection to the corresponding `GraphNode` in the
    /// second collection.
    ///
    /// # Rules
    /// * The first collection must have the same number of `GraphNode`s as the second collection.
    /// * the `GraphNode`'s in the second collection must have an arity of either `Any` or `Exact(1)`.
    OneToOne,
    /// Connects each `GraphNode` in the first collection to all `GraphNode`s in the second collection.
    ///
    /// # Rules
    /// * The second collection must be of size `n` where `n` is a multiple of the size of the first
    ///   collection IE: `n % first.len() == 0`.
    /// * The `GraphNode`'s in the second collection must have an arity of either `Any` or `Exact(n / first.len())`.
    OneToMany,
    /// Connects all `GraphNode`s in the first collection to a single `GraphNode` in the second collection.
    ///
    /// # Rules
    /// * The first collection must be of size `n` where `n` is a multiple of the size of the second collection
    ///   IE: `n % second.len() == 0`.
    /// * The `GraphNode`'s in the first collection must have an arity of either `Any` or `Exact(n / second.len())`.
    ManyToOne,
    /// Connects all `GraphNode`s in the first collection to all `GraphNode`s in the second collection.
    ///
    /// # Rules
    /// * The `GraphNode`'s in the second collection must have an arity of either `Any` or `Exact(n)`
    ///   where `n` is the size of the first collection.
    AllToAll,
}

/// Represents a relationship between two [GraphNode]'s where the `source_id` is the [GraphNode]'s
/// id that is incoming, or giving its value to the `target_id` [GraphNode].
struct Relationship<'a> {
    source_id: &'a GraphNodeId,
    target_id: &'a GraphNodeId,
}

/// The [GraphAggregate] struct is a builder for [Graph] that allows you to build a [Graph]
/// in an declarative way. It allows you to build a [Graph] by connecting [GraphNode]'s together in
/// a multitude of ways.
///
/// We do this by maintaining a map of nodes with an aggregate of thir relationships or how they are
/// connected to each other. Because of the nature of this aggregate, verifying the correctness of a
/// [Graph] can truly only be done after building the final [Graph].
#[derive(Default)]
pub struct GraphAggregate<'a, T: Clone> {
    nodes: BTreeMap<&'a GraphNodeId, &'a GraphNode<T>>,
    node_order: BTreeMap<usize, &'a GraphNodeId>,
    relationships: Vec<Relationship<'a>>,
}

impl<'a, T: Clone> GraphAggregate<'a, T> {
    pub fn new() -> Self {
        GraphAggregate {
            nodes: BTreeMap::new(),
            node_order: BTreeMap::new(),
            relationships: Vec::new(),
        }
    }

    pub fn build(&self) -> Graph<T> {
        let mut id_index_map = BTreeMap::new();
        let mut graph = Graph::<T>::default();

        for (index, node_id) in self.node_order.values().enumerate() {
            let node = self.nodes[node_id];

            graph.push((index, node.node_type(), node.value().clone(), node.arity()));
            id_index_map.insert(node_id, index);
        }

        for rel in self.relationships.iter() {
            let source_idx = id_index_map[&rel.source_id];
            let target_idx = id_index_map[&rel.target_id];

            graph.attach(source_idx, target_idx);
        }

        graph.set_cycles(vec![]);
        graph
    }

    pub fn cycle<G: AsRef<[GraphNode<T>]>>(self, one: &'a G) -> Self {
        self.one_to_one(one, one)
    }

    pub fn one_to_one<G: AsRef<[GraphNode<T>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::OneToOne, one, two);
        self
    }

    pub fn one_to_many<G: AsRef<[GraphNode<T>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::OneToMany, one, two);
        self
    }

    pub fn many_to_one<G: AsRef<[GraphNode<T>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::ManyToOne, one, two);
        self
    }

    pub fn all_to_all<G: AsRef<[GraphNode<T>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::AllToAll, one, two);
        self
    }

    pub fn insert<G: AsRef<[GraphNode<T>]>>(mut self, collection: &'a G) -> Self {
        self.attach(collection.as_ref());
        self
    }

    pub fn layer<G: AsRef<[GraphNode<T>]>>(&self, collections: Vec<&'a G>) -> Self {
        let mut conn = GraphAggregate::new();
        let mut previous = collections[0];

        for collection in collections.iter() {
            conn.attach((*collection).as_ref());
        }

        for coll in collections.iter().skip(1) {
            conn = conn.one_to_one(previous, coll);
            previous = coll;
        }

        conn
    }

    fn attach(&mut self, group: &'a [GraphNode<T>]) {
        for node in group.iter() {
            let node_id = node.id();

            if !self.nodes.contains_key(node_id) {
                self.nodes.insert(node_id, node);
                self.node_order.insert(self.node_order.len(), node_id);

                group
                    .iter()
                    .filter(|item| node.outgoing().contains(&item.index()))
                    .for_each(|item| {
                        self.relationships.push(Relationship {
                            source_id: node.id(),
                            target_id: item.id(),
                        });
                    });
            }
        }
    }

    fn connect<G: AsRef<[GraphNode<T>]>>(
        &mut self,
        connection: ConnectTypes,
        one: &'a G,
        two: &'a G,
    ) {
        self.attach(one.as_ref());
        self.attach(two.as_ref());

        match connection {
            ConnectTypes::OneToOne => self.one_to_one_connect(one, two),
            ConnectTypes::OneToMany => self.one_to_many_connect(one, two),
            ConnectTypes::ManyToOne => self.many_to_one_connect(one, two),
            ConnectTypes::AllToAll => self.all_to_all_connect(one, two),
        }
    }

    fn one_to_one_connect<G: AsRef<[GraphNode<T>]>>(&mut self, one: &'a G, two: &'a G) {
        let one_outputs = Self::get_outputs(one);
        let two_inputs = Self::get_inputs(two);

        if one_outputs.len() != two_inputs.len() {
            panic!("OneToOne - oneGroup outputs must be the same length as twoGroup inputs.");
        }

        for (one, two) in one_outputs.into_iter().zip(two_inputs.into_iter()) {
            self.relationships.push(Relationship {
                source_id: one.id(),
                target_id: two.id(),
            });
        }
    }

    fn one_to_many_connect<G: AsRef<[GraphNode<T>]>>(&mut self, one: &'a G, two: &'a G) {
        let one_outputs = Self::get_outputs(one);
        let two_inputs = Self::get_inputs(two);

        if two_inputs.len() % one_outputs.len() != 0 {
            panic!("OneToMany - TwoGroup inputs must be a multiple of OneGroup outputs.");
        }

        for targets in two_inputs.chunks(one_outputs.len()) {
            for (source, target) in one_outputs.iter().zip(targets.iter()) {
                self.relationships.push(Relationship {
                    source_id: source.id(),
                    target_id: target.id(),
                });
            }
        }
    }

    fn many_to_one_connect<G: AsRef<[GraphNode<T>]>>(&mut self, one: &'a G, two: &'a G) {
        let one_outputs = Self::get_outputs(one);
        let two_inputs = Self::get_inputs(two);

        if one_outputs.len() % two_inputs.len() != 0 {
            panic!("ManyToOne - OneGroup outputs must be a multiple of TwoGroup inputs.");
        }

        for sources in one_outputs.chunks(two_inputs.len()) {
            for (source, target) in sources.iter().zip(two_inputs.iter()) {
                self.relationships.push(Relationship {
                    source_id: source.id(),
                    target_id: target.id(),
                });
            }
        }
    }

    fn all_to_all_connect<G: AsRef<[GraphNode<T>]>>(&mut self, one: &'a G, two: &'a G) {
        let one_outputs = Self::get_outputs(one);
        let two_inputs = Self::get_inputs(two);

        for source in one_outputs {
            for target in two_inputs.iter() {
                self.relationships.push(Relationship {
                    source_id: source.id(),
                    target_id: target.id(),
                });
            }
        }
    }

    fn get_outputs<G: AsRef<[GraphNode<T>]>>(collection: &'a G) -> Vec<&'a GraphNode<T>> {
        let outputs = collection
            .as_ref()
            .iter()
            .enumerate()
            .skip_while(|(_, node)| !node.outgoing().is_empty())
            .filter_map(|(idx, _)| collection.as_ref().get(idx))
            .collect::<Vec<&GraphNode<T>>>();

        if !outputs.is_empty() {
            return outputs;
        }

        let recurrent_outputs = collection
            .as_ref()
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                node.outgoing().len() == 1
                    && node.is_recurrent()
                    && node.node_type() == NodeType::Vertex
            })
            .filter_map(|(idx, _)| collection.as_ref().get(idx))
            .collect::<Vec<&GraphNode<T>>>();

        if !recurrent_outputs.is_empty() {
            return recurrent_outputs;
        }

        collection
            .as_ref()
            .iter()
            .enumerate()
            .filter(|(_, node)| node.incoming().is_empty())
            .filter_map(|(idx, _)| collection.as_ref().get(idx))
            .collect::<Vec<&GraphNode<T>>>()
    }

    fn get_inputs<G: AsRef<[GraphNode<T>]>>(collection: &'a G) -> Vec<&'a GraphNode<T>> {
        let inputs = collection
            .as_ref()
            .iter()
            .enumerate()
            .take_while(|(_, node)| node.incoming().is_empty())
            .filter_map(|(idx, _)| collection.as_ref().get(idx))
            .collect::<Vec<&GraphNode<T>>>();

        if !inputs.is_empty() {
            return inputs;
        }

        let recurrent_inputs = collection
            .as_ref()
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                node.outgoing().len() == 1
                    && node.is_recurrent()
                    && node.node_type() == NodeType::Vertex
            })
            .filter_map(|(idx, _)| collection.as_ref().get(idx))
            .collect::<Vec<&GraphNode<T>>>();

        if !recurrent_inputs.is_empty() {
            return recurrent_inputs;
        }

        collection
            .as_ref()
            .iter()
            .enumerate()
            .filter(|(_, node)| node.outgoing().is_empty())
            .filter_map(|(idx, _)| collection.as_ref().get(idx))
            .collect::<Vec<&GraphNode<T>>>()
    }
}

#[cfg(test)]
mod tests {
    use crate::{GraphAggregate, GraphNode, Node, NodeType};
    use radiate_core::Valid;
    use std::panic;

    fn should_panic<F: FnOnce() -> () + panic::UnwindSafe>(f: F) {
        assert!(panic::catch_unwind(f).is_err());
    }

    #[test]
    fn test_graph_aggregate_one_to_one() {
        let graph = GraphAggregate::new()
            .one_to_one(
                &vec![
                    GraphNode::new(0, NodeType::Input, 0),
                    GraphNode::new(1, NodeType::Input, 1),
                ],
                &vec![
                    GraphNode::new(0, NodeType::Output, 0),
                    GraphNode::new(1, NodeType::Output, 1),
                ],
            )
            .build();

        let input_nodes = graph.inputs();
        let output_nodes = graph.outputs();

        let input_one = &graph[0];
        let input_two = &graph[1];
        let output_one = &graph[2];
        let output_two = &graph[3];

        assert_eq!(input_one.index(), 0);
        assert_eq!(input_two.index(), 1);
        assert_eq!(output_one.index(), 2);
        assert_eq!(output_two.index(), 3);

        for (in_node, out_node) in input_nodes.zip(output_nodes) {
            assert!(in_node.outgoing().contains(&out_node.index()));
            assert!(out_node.incoming().contains(&in_node.index()));

            assert_eq!(in_node.outgoing().len(), 1);
            assert_eq!(in_node.incoming().len(), 0);
            assert_eq!(out_node.incoming().len(), 1);
            assert_eq!(out_node.outgoing().len(), 0);
        }

        assert!(graph.is_valid());
    }

    #[test]
    fn test_graph_aggregate_one_to_many() {
        let graph = GraphAggregate::new()
            .one_to_many(
                &vec![
                    GraphNode::new(0, NodeType::Input, 0),
                    GraphNode::new(1, NodeType::Input, 1),
                ],
                &vec![
                    GraphNode::new(2, NodeType::Output, 0),
                    GraphNode::new(3, NodeType::Output, 1),
                    GraphNode::new(4, NodeType::Output, 2),
                    GraphNode::new(5, NodeType::Output, 3),
                ],
            )
            .build();

        let input_one = &graph[0];
        let input_two = &graph[1];

        let output_one = &graph[2];
        let output_two = &graph[3];
        let output_three = &graph[4];
        let output_four = &graph[5];

        assert!(input_one.outgoing().contains(&output_one.index()));
        assert!(input_one.outgoing().contains(&output_three.index()));
        assert!(input_two.outgoing().contains(&output_two.index()));
        assert!(input_two.outgoing().contains(&output_four.index()));
        assert!(output_one.incoming().contains(&input_one.index()));
        assert!(output_two.incoming().contains(&input_two.index()));
        assert!(output_three.incoming().contains(&input_one.index()));
        assert!(output_four.incoming().contains(&input_two.index()));

        assert_eq!(input_one.outgoing().len(), 2);
        assert_eq!(input_two.outgoing().len(), 2);
        assert_eq!(output_one.incoming().len(), 1);
        assert_eq!(output_two.incoming().len(), 1);
        assert_eq!(output_three.incoming().len(), 1);
        assert_eq!(output_four.incoming().len(), 1);

        assert!(graph.is_valid());
    }

    #[test]
    fn test_graph_aggregate_many_to_one() {
        let graph = GraphAggregate::new()
            .many_to_one(
                &vec![
                    GraphNode::new(0, NodeType::Input, 0),
                    GraphNode::new(123, NodeType::Input, 1),
                    GraphNode::new(111, NodeType::Input, 2),
                    GraphNode::new(3, NodeType::Input, 3),
                ],
                &vec![
                    GraphNode::new(10000, NodeType::Output, 0),
                    GraphNode::new(123, NodeType::Output, 1),
                ],
            )
            .build();

        let input_one = &graph[0];
        let input_two = &graph[1];
        let input_three = &graph[2];
        let input_four = &graph[3];

        let output_one = &graph[4];
        let output_two = &graph[5];

        assert_eq!(input_one.index(), 0);
        assert_eq!(input_two.index(), 1);
        assert_eq!(input_three.index(), 2);
        assert_eq!(input_four.index(), 3);
        assert_eq!(output_one.index(), 4);
        assert_eq!(output_two.index(), 5);

        assert!(input_one.outgoing().contains(&output_one.index()));
        assert!(input_three.outgoing().contains(&output_one.index()));
        assert!(input_two.outgoing().contains(&output_two.index()));
        assert!(input_four.outgoing().contains(&output_two.index()));
        assert!(output_one.incoming().contains(&input_one.index()));
        assert!(output_one.incoming().contains(&input_three.index()));
        assert!(output_two.incoming().contains(&input_two.index()));
        assert!(output_two.incoming().contains(&input_four.index()));

        assert_eq!(input_one.outgoing().len(), 1);
        assert_eq!(input_two.outgoing().len(), 1);
        assert_eq!(input_three.outgoing().len(), 1);
        assert_eq!(input_four.outgoing().len(), 1);
        assert_eq!(output_one.incoming().len(), 2);
        assert_eq!(output_two.incoming().len(), 2);

        assert!(graph.is_valid());
    }

    #[test]
    fn test_graph_aggregate_all_to_all() {
        let graph = GraphAggregate::new()
            .all_to_all(
                &vec![
                    GraphNode::new(0, NodeType::Input, 0),
                    GraphNode::new(1, NodeType::Input, 1),
                ],
                &vec![
                    GraphNode::new(2, NodeType::Output, 0),
                    GraphNode::new(3, NodeType::Output, 1),
                    GraphNode::new(4, NodeType::Output, 2),
                ],
            )
            .build();

        let input_one = &graph[0];
        let input_two = &graph[1];

        let output_one = &graph[2];
        let output_two = &graph[3];
        let output_three = &graph[4];

        assert!(input_one.outgoing().contains(&output_one.index()));
        assert!(input_one.outgoing().contains(&output_two.index()));
        assert!(input_one.outgoing().contains(&output_three.index()));
        assert!(input_two.outgoing().contains(&output_one.index()));
        assert!(input_two.outgoing().contains(&output_two.index()));
        assert!(input_two.outgoing().contains(&output_three.index()));

        assert!(output_one.incoming().contains(&input_one.index()));
        assert!(output_one.incoming().contains(&input_two.index()));
        assert!(output_two.incoming().contains(&input_one.index()));
        assert!(output_two.incoming().contains(&input_two.index()));
        assert!(output_three.incoming().contains(&input_one.index()));
        assert!(output_three.incoming().contains(&input_two.index()));

        assert_eq!(input_one.outgoing().len(), 3);
        assert_eq!(input_two.outgoing().len(), 3);
        assert_eq!(output_one.incoming().len(), 2);
        assert_eq!(output_two.incoming().len(), 2);
        assert_eq!(output_three.incoming().len(), 2);

        assert!(graph.is_valid());
    }

    #[test]
    fn test_graph_aggregate_cycle() {
        let graph = GraphAggregate::new()
            .cycle(&vec![
                GraphNode::new(0, NodeType::Vertex, 0),
                GraphNode::new(1, NodeType::Vertex, 1),
            ])
            .build();

        let node_one = &graph[0];
        let node_two = &graph[1];

        assert!(node_one.outgoing().contains(&node_one.index()));
        assert!(node_one.incoming().contains(&node_one.index()));
        assert!(node_one.is_recurrent());
        assert!(node_one.node_type() == NodeType::Vertex);

        assert!(node_two.outgoing().contains(&node_two.index()));
        assert!(node_two.incoming().contains(&node_two.index()));
        assert!(node_two.is_recurrent());
        assert!(node_two.node_type() == NodeType::Vertex);

        assert!(graph.is_valid());
    }

    #[test]
    fn test_graph_aggregate_layer() {
        let graph = GraphAggregate::new()
            .layer(vec![
                &vec![
                    GraphNode::new(0, NodeType::Input, "ONE"),
                    GraphNode::new(1, NodeType::Input, "TWO"),
                ],
                &vec![
                    GraphNode::new(2, NodeType::Vertex, "THREE"),
                    GraphNode::new(3, NodeType::Vertex, "FOUR"),
                ],
                &vec![
                    GraphNode::new(4, NodeType::Output, "FIVE"),
                    GraphNode::new(5, NodeType::Output, "SIX"),
                ],
            ])
            .build();

        assert!(graph.is_valid());

        let input_one = &graph[0];
        let input_two = &graph[1];
        let vertex_one = &graph[2];
        let vertex_two = &graph[3];
        let output_one = &graph[4];
        let output_two = &graph[5];

        assert!(input_one.outgoing().contains(&vertex_one.index()));
        assert!(input_two.outgoing().contains(&vertex_two.index()));
        assert!(vertex_one.incoming().contains(&input_one.index()));
        assert!(vertex_two.incoming().contains(&input_two.index()));
        assert!(vertex_one.outgoing().contains(&output_one.index()));
        assert!(vertex_two.outgoing().contains(&output_two.index()));
        assert!(output_one.incoming().contains(&vertex_one.index()));
        assert!(output_two.incoming().contains(&vertex_two.index()));
    }

    #[test]
    fn one_to_one_size_mismatch_panics() {
        should_panic(|| {
            let _ = GraphAggregate::new()
                .one_to_one(
                    &vec![GraphNode::new(0, NodeType::Input, 0)],
                    &vec![
                        GraphNode::new(1, NodeType::Output, 0),
                        GraphNode::new(2, NodeType::Output, 1),
                    ],
                )
                .build();
        });
    }

    #[test]
    fn one_to_many_invalid_multiple_panics() {
        should_panic(|| {
            let _ = GraphAggregate::new()
                .one_to_many(
                    &vec![
                        GraphNode::new(0, NodeType::Input, 0),
                        GraphNode::new(1, NodeType::Input, 1),
                    ],
                    &vec![
                        // 3 is NOT a multiple of 2 -> should panic
                        GraphNode::new(2, NodeType::Output, 0),
                        GraphNode::new(3, NodeType::Output, 1),
                        GraphNode::new(4, NodeType::Output, 2),
                    ],
                )
                .build();
        });
    }

    #[test]
    fn many_to_one_invalid_multiple_panics() {
        should_panic(|| {
            let _ = GraphAggregate::new()
                .many_to_one(
                    &vec![
                        GraphNode::new(0, NodeType::Input, 0),
                        GraphNode::new(1, NodeType::Input, 1),
                        GraphNode::new(2, NodeType::Input, 2),
                    ],
                    &vec![
                        // 3 (one) % 2 (two) != 0 -> panic
                        GraphNode::new(10, NodeType::Output, 0),
                        GraphNode::new(11, NodeType::Output, 1),
                    ],
                )
                .build();
        });
    }

    #[test]
    fn duplicate_insert_does_not_duplicate_nodes_or_edges() {
        let inputs = vec![
            GraphNode::new(0, NodeType::Input, "A"),
            GraphNode::new(1, NodeType::Input, "B"),
        ];
        let outputs = vec![
            GraphNode::new(2, NodeType::Output, "C"),
            GraphNode::new(3, NodeType::Output, "D"),
        ];

        let graph = GraphAggregate::new()
            .insert(&inputs) // first insert
            .insert(&inputs) // duplicate insert
            .one_to_one(&inputs, &outputs)
            .build();

        // Expect 4 nodes only
        assert_eq!(graph.len(), 4);

        // And exactly the two one-to-one edges
        let out0 = &graph[0].outgoing();
        let out1 = &graph[1].outgoing();
        assert_eq!(out0.len(), 1);
        assert_eq!(out1.len(), 1);
        assert!(out0.contains(&2));
        assert!(out1.contains(&3));
    }

    #[test]
    fn node_index_order_is_deterministic_by_insert_order() {
        // Insert order: inputs then outputs; indices should follow that order.
        let inputs = vec![
            GraphNode::new(10, NodeType::Input, "i0"),
            GraphNode::new(11, NodeType::Input, "i1"),
        ];
        let outputs = vec![
            GraphNode::new(20, NodeType::Output, "o0"),
            GraphNode::new(21, NodeType::Output, "o1"),
        ];

        let g1 = GraphAggregate::new()
            .insert(&inputs)
            .insert(&outputs)
            .build();

        assert_eq!(g1[0].node_type(), NodeType::Input);
        assert_eq!(g1[1].node_type(), NodeType::Input);
        assert_eq!(g1[2].node_type(), NodeType::Output);
        assert_eq!(g1[3].node_type(), NodeType::Output);
    }

    #[test]
    fn get_inputs_fallback_paths_are_respected() {
        // Make outputs with no incoming/outgoing to trigger input-fallback
        let outs = vec![
            GraphNode::new(100, NodeType::Output, "o0"),
            GraphNode::new(101, NodeType::Output, "o1"),
        ];

        let g = GraphAggregate::new()
            .one_to_one(
                &vec![
                    GraphNode::new(0, NodeType::Input, "i0"),
                    GraphNode::new(1, NodeType::Input, "i1"),
                ],
                &outs,
            )
            .build();

        // Inputs 0..1 connect to 2..3
        assert!(g[0].outgoing().contains(&2));
        assert!(g[1].outgoing().contains(&3));
        assert!(g[2].incoming().contains(&0));
        assert!(g[3].incoming().contains(&1));
        assert!(g.is_valid());
    }

    #[test]
    fn all_to_all_edge_counts_match() {
        let ins = vec![
            GraphNode::new(0, NodeType::Input, 0),
            GraphNode::new(1, NodeType::Input, 1),
            GraphNode::new(2, NodeType::Input, 2),
        ];
        let outs = vec![
            GraphNode::new(3, NodeType::Output, 0),
            GraphNode::new(4, NodeType::Output, 1),
        ];

        let g = GraphAggregate::new().all_to_all(&ins, &outs).build();

        // Each input connects to all outputs
        for i in 0..ins.len() {
            assert_eq!(g[i].outgoing().len(), outs.len());
        }
        // Each output receives from all inputs
        for j in 0..outs.len() {
            assert_eq!(g[ins.len() + j].incoming().len(), ins.len());
        }
        assert!(g.is_valid());
    }

    #[test]
    fn cycle_builds_self_loops_only() {
        let verts = vec![
            GraphNode::new(0, NodeType::Vertex, 0),
            GraphNode::new(1, NodeType::Vertex, 1),
            GraphNode::new(2, NodeType::Vertex, 2),
        ];

        let g = GraphAggregate::new().cycle(&verts).build();

        for i in 0..verts.len() {
            assert!(g[i].outgoing().contains(&i));
            assert!(g[i].incoming().contains(&i));
            assert_eq!(g[i].outgoing().len(), 1);
            assert_eq!(g[i].incoming().len(), 1);
        }
        assert!(g.is_valid());
    }

    #[test]
    fn one_to_many_chunking_is_correct() {
        // 2 inputs -> 6 outputs (chunks of 2)
        let ins = vec![
            GraphNode::new(0, NodeType::Input, "a"),
            GraphNode::new(1, NodeType::Input, "b"),
        ];
        let outs = vec![
            GraphNode::new(2, NodeType::Output, "o0"),
            GraphNode::new(3, NodeType::Output, "o1"),
            GraphNode::new(4, NodeType::Output, "o2"),
            GraphNode::new(5, NodeType::Output, "o3"),
            GraphNode::new(6, NodeType::Output, "o4"),
            GraphNode::new(7, NodeType::Output, "o5"),
        ];

        let g = GraphAggregate::new().one_to_many(&ins, &outs).build();

        // Expect pairs: (0->2,1->3),(0->4,1->5),(0->6,1->7)
        for (_, chunk_start) in (2..=6).step_by(2).enumerate() {
            let i0 = 0;
            let i1 = 1;
            let o0 = chunk_start;
            let o1 = chunk_start + 1;

            assert!(g[i0].outgoing().contains(&o0));
            assert!(g[i1].outgoing().contains(&o1));
            assert!(g[o0].incoming().contains(&i0));
            assert!(g[o1].incoming().contains(&i1));
        }
        assert!(g.is_valid());
    }

    #[test]
    fn many_to_one_chunking_is_correct() {
        // 6 inputs -> 2 outputs (chunks of 2)
        let ins = vec![
            GraphNode::new(0, NodeType::Input, "a"),
            GraphNode::new(1, NodeType::Input, "b"),
            GraphNode::new(2, NodeType::Input, "c"),
            GraphNode::new(3, NodeType::Input, "d"),
            GraphNode::new(4, NodeType::Input, "e"),
            GraphNode::new(5, NodeType::Input, "f"),
        ];
        let outs = vec![
            GraphNode::new(6, NodeType::Output, "o0"),
            GraphNode::new(7, NodeType::Output, "o1"),
        ];

        let g = GraphAggregate::new().many_to_one(&ins, &outs).build();

        // Expect pairs of inputs mapping to both outputs by chunks of outs.len()
        // Chunks: [0,1] -> [6,7]; [2,3] -> [6,7]; [4,5] -> [6,7]
        for pair in &[(0, 1), (2, 3), (4, 5)] {
            assert!(g[pair.0].outgoing().contains(&6));
            assert!(g[pair.1].outgoing().contains(&7));
            assert!(g[6].incoming().contains(&pair.0));
            assert!(g[7].incoming().contains(&pair.1));
        }
        assert!(g.is_valid());
    }
}
