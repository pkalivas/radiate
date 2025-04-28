use super::node::GraphNodeId;
use crate::{
    collections::{Graph, GraphNode, NodeType},
    node::Node,
};
use std::collections::BTreeMap;

/// Building a [Graph<T>] can be a very complex task. Everything in this file exists
/// to simplify the process of building a [Graph<T>] by allowing the user to do so
/// in a declarative way.
///
/// The [ConnectTypes] are simply a set of available ways we can
/// connect different [GraphNode]'s together.
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
    /// Connects each `GraphNode` in the first collection to the corresponding `GraphNode` in the second collection
    /// then connects each `GraphNode` in the second collection to the corresponding `GraphNode` in the first collection.
    /// This creates a self-referential relationship between the two collections - IE: a cycle.
    ///
    /// # Rules
    /// * The first collection must have the same number of `GraphNode`s as the second collection.
    /// * the `GraphNode`'s in the first collection must have an arity of either `Any` or `Exact(2)`.
    /// * the `GraphNode`'s in the second collection must have an arity of either `Any` or `Exact(1)`.
    OneToSelf,
}

/// Represents a relationship between two `GraphNode`s where the `source_id` is the [GraphNode<T>]'s
/// id that is incoming, or giving its value to the `target_id` [GraphNode<T>].
struct Relationship<'a> {
    source_id: &'a GraphNodeId,
    target_id: &'a GraphNodeId,
}

/// The [GraphAggregate] struct is a builder for [Graph<T>] that allows you to build a [Graph<T>]
/// in an extremely declarative way. It allows you to build a [Graph<T>] by connecting
/// [GraphNode]'s together in all sorts of ways. This results in an extremely powerful tool.
/// The [GraphAggregate] is ment to take a collection of `GraphNode`s and connect them together
/// in a sudo 'layered' way. I say 'sudo' because the 'layers' can simply be connecting
/// input nodes to output nodes, hidden nodes to weights, input nodes to output nodes, recurrent
/// connections, etc.
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
            let node = self.nodes.get(node_id).unwrap();

            graph.push((index, node.node_type(), node.value().clone(), node.arity()));
            id_index_map.insert(node_id, index);
        }

        for rel in self.relationships.iter() {
            let source_idx = id_index_map.get(&rel.source_id).unwrap();
            let target_idx = id_index_map.get(&rel.target_id).unwrap();

            graph.attach(*source_idx, *target_idx);
        }

        graph.set_cycles(vec![]);
        graph
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

    pub fn one_to_self<G: AsRef<[GraphNode<T>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::OneToSelf, one, two);
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
            ConnectTypes::OneToSelf => self.one_to_self_connect(one, two),
        }
    }

    fn one_to_one_connect<G: AsRef<[GraphNode<T>]>>(&mut self, one: &'a G, two: &'a G) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

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
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

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
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

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
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        for source in one_outputs {
            for target in two_inputs.iter() {
                self.relationships.push(Relationship {
                    source_id: source.id(),
                    target_id: target.id(),
                });
            }
        }
    }

    fn one_to_self_connect<G: AsRef<[GraphNode<T>]>>(&mut self, one: &'a G, two: &'a G) {
        let one_outputs = self.get_outputs(one);
        let two_inputs = self.get_inputs(two);

        if one_outputs.len() != two_inputs.len() {
            panic!("Self - oneGroup outputs must be the same length as twoGroup inputs.");
        }

        for (one, two) in one_outputs.into_iter().zip(two_inputs.into_iter()) {
            self.relationships.push(Relationship {
                source_id: one.id(),
                target_id: two.id(),
            });
            self.relationships.push(Relationship {
                source_id: two.id(),
                target_id: one.id(),
            });
        }
    }

    fn get_outputs<G: AsRef<[GraphNode<T>]>>(&self, collection: &'a G) -> Vec<&'a GraphNode<T>> {
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
                    && (node.node_type() == NodeType::Vertex)
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

    fn get_inputs<G: AsRef<[GraphNode<T>]>>(&self, collection: &'a G) -> Vec<&'a GraphNode<T>> {
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
    use crate::{GraphAggregate, GraphNode, NodeType};
    use radiate_core::Valid;

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

        for (in_node, out_node) in input_nodes.iter().zip(output_nodes.iter()) {
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
}

// let mut graph = Graph::<T>::default();

// let res = graph.try_modify(|mut trans| {
//     let mut node_id_index_map = BTreeMap::new();

//     for (index, node_id) in self.node_order.values().enumerate() {
//         let node = self.nodes.get(node_id).unwrap();

//         trans.add_node((index, node.node_type(), node.value().clone(), node.arity()));
//         node_id_index_map.insert(node_id, index);
//     }

//     for rel in self.relationships.iter() {
//         let source_idx = node_id_index_map.get(&rel.source_id).unwrap();
//         let target_idx = node_id_index_map.get(&rel.target_id).unwrap();

//         trans.attach(*source_idx, *target_idx);
//     }

//     trans.set_cycles();
//     trans.commit()
// });

// match res {
//     TransactionResult::Valid(_) => graph,
//     _ => panic!("Graph is not valid"),
// }
