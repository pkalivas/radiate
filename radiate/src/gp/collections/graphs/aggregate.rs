use super::node::GraphNodeId;
use crate::{
    collections::{Graph, GraphNode, NodeType},
    node::Node,
};
use std::collections::BTreeMap;

/// Building a `Graph<T>` can be a very complex task. Everything in this file exists
/// to simplify the process of building a `Graph<T>` by allowing the user to do so
/// in a declarative way.
///
/// The `ConnectTypes` are simply a set of available ways we can
/// connect different `GraphNode`s together.
///
/// # Assumptions
/// * The first collection is the 'source' collection and the second collection is the 'target' collection.
/// * The target collection's `GraphNode`'s `Arity` is compatible with the `ConnectTypes`.
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

/// Represents a relationship between two `GraphNode`s where the `source_id` is the `GraphNode<T>`'s
/// id that is incoming, or giving its value to the `target_id` `GraphNode<T>`.
struct Relationship<'a> {
    source_id: &'a GraphNodeId,
    target_id: &'a GraphNodeId,
}

/// The `GraphArchitect` struct is a builder for `Graph<T>` that allows you to build a `Graph<T>`
/// in an extremely declarative way. It allows you to build a `Graph<T>` by connecting
/// `GraphNode`s together in all sorts of ways. This results in an extremely powerful tool.
/// The `GraphArchitect` is ment to take a collection of `GraphNode`s and connect them together
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

    /// Builds the `Graph<T>` from the `GraphAggregate<T>`.
    /// This method will take the `GraphAggregate<T>` and build a `Graph<T>` from it.
    /// It will use the relationships between the `GraphNode`s to connect them together in the
    /// resulting graph in the desired way. Because we keep track of nodes without having them fully
    /// initialized (meaning the nodes themselves are not connected yet) we first have to build the nodes by
    /// copying them into a new `GraphNode<T>` and then connecting them together using the relationships.
    ///
    /// The upside for this is that we can build a `Graph<T>` in a very declarative and large way,
    /// enabling users to really create whatever type of graph they want. However, on the downside,
    /// it can sometimes be hard to keep ensure that a given node has a correct arity, or that the
    /// relationships are correct. It is always advantagous to ensure a `Graph<T>` is valid after
    /// building it and to follow the rules of the `ConnectTypes` when connecting nodes together.
    ///
    /// Usually the `GraphNode`'s Index is extremely important as it represents the position of the
    /// node within the `Graph<T>`. However, when building a `Graph<T>` from a `GraphAggregate<T>`,
    /// the index is not important as the `Graph<T>` will re-index the nodes when it is built.
    ///
    /// # Returns
    /// A `Graph<T>` that has been built from the `GraphAggregate<T>`.
    pub fn build(&self) -> Graph<T> {
        let mut node_id_index_map = BTreeMap::new();
        let mut graph = Graph::<T>::default();

        for (index, node_id) in self.node_order.values().enumerate() {
            let node = self.nodes.get(node_id).unwrap();

            graph.push((index, node.node_type(), node.value().clone(), node.arity()));
            node_id_index_map.insert(node_id, index);
        }

        for rel in self.relationships.iter() {
            let source_idx = node_id_index_map.get(&rel.source_id).unwrap();
            let target_idx = node_id_index_map.get(&rel.target_id).unwrap();

            graph.attach(*source_idx, *target_idx);
        }

        graph.set_cycles(vec![]);
        graph
    }

    /// Connects the `GraphNode`s in the first collection to the `GraphNode`s in the second collection
    /// in a one-to-one relationship.
    ///
    /// # Example
    /// ```
    /// use radiate::*;
    ///
    /// let source_nodes = vec![
    ///     GraphNode::new(0, NodeType::Input, 0),
    ///     GraphNode::new(1, NodeType::Input, 1),
    /// ];
    ///
    /// let target_nodes = vec![
    ///     GraphNode::new(0, NodeType::Output, 0),
    ///     GraphNode::new(1, NodeType::Output, 1),
    /// ];
    ///
    /// let mut graph = GraphAggregate::new()
    ///     .one_to_one(&source_nodes, &target_nodes)
    ///     .build();
    ///
    /// assert!(graph.is_valid());
    /// ```
    /// This will create a `Graph<T>` that looks like this:
    /// ``` text
    /// 0 -> 2
    /// 1 -> 3
    /// ```
    ///
    /// # Arguments
    /// * `one`: The first collection of `GraphNode`s.
    /// * `two`: The second collection of `GraphNode`s.
    pub fn one_to_one<G: AsRef<[GraphNode<T>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::OneToOne, one, two);
        self
    }

    /// Connects the `GraphNode`s in the first collection to the `GraphNode`s in the second collection
    /// in a one-to-many relationship.
    ///
    /// # Example
    /// ```
    /// use radiate::*;
    ///
    /// let source_nodes = vec![
    ///     GraphNode::new(0, NodeType::Input, 0),
    ///     GraphNode::new(1, NodeType::Input, 1),
    /// ];
    ///
    /// let target_nodes = vec![
    ///     GraphNode::new(0, NodeType::Output, 0),
    ///     GraphNode::new(1, NodeType::Output, 1),
    ///     GraphNode::new(2, NodeType::Output, 2),
    ///     GraphNode::new(3, NodeType::Output, 3),
    /// ];
    ///
    /// let mut graph = GraphAggregate::new()
    ///    .one_to_many(&source_nodes, &target_nodes)
    ///    .build();
    ///
    /// assert!(graph.is_valid());
    /// ```
    ///
    /// This will create a `Graph<T>` that looks like this:
    /// ``` text
    /// 0 -> [2, 3]
    /// 1 -> [4, 5]
    /// ```
    ///
    /// # Arguments
    /// * `one`: The first collection of `GraphNode`s.
    /// * `two`: The second collection of `GraphNode`s.
    pub fn one_to_many<G: AsRef<[GraphNode<T>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::OneToMany, one, two);
        self
    }

    /// Connects the `GraphNode`s in the first collection to the `GraphNode`s in the second collection
    /// in a many-to-one relationship.
    ///
    /// # Example
    /// ```
    /// use radiate::*;
    ///
    /// let source_nodes = vec![
    ///     GraphNode::new(0, NodeType::Input, 0),
    ///     GraphNode::new(1, NodeType::Input, 1),
    ///     GraphNode::new(2, NodeType::Input, 2),
    ///     GraphNode::new(3, NodeType::Input, 3),
    /// ];
    ///
    /// let target_nodes = vec![
    ///     GraphNode::new(0, NodeType::Output, 0),
    ///     GraphNode::new(1, NodeType::Output, 1),
    /// ];
    ///
    /// let mut graph = GraphAggregate::new()
    ///     .many_to_one(&source_nodes, &target_nodes)
    ///     .build();
    ///
    /// assert!(graph.is_valid());
    /// ```
    ///
    /// This will create a `Graph<T>` that looks like this:
    /// ``` text
    /// [0, 1] -> 4
    /// [2, 3] -> 5
    /// ```
    ///
    /// # Arguments
    /// * `one`: The first collection of `GraphNode`s.
    /// * `two`: The second collection of `GraphNode`s.
    pub fn many_to_one<G: AsRef<[GraphNode<T>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::ManyToOne, one, two);
        self
    }

    /// Connects the `GraphNode`s in the first collection to the `GraphNode`s in the second collection
    /// in an all-to-all relationship. This means that each `GraphNode` in the first collection will be connected to each `GraphNode`
    /// in the second collection.
    ///
    /// # Example
    /// ```
    /// use radiate::*;
    ///
    /// let source_nodes = vec![
    ///     GraphNode::new(0, NodeType::Input, 0),
    ///     GraphNode::new(1, NodeType::Input, 1),
    /// ];
    ///
    /// let target_nodes = vec![
    ///   GraphNode::new(0, NodeType::Output, 0),
    ///   GraphNode::new(1, NodeType::Output, 1),
    ///   GraphNode::new(2, NodeType::Output, 2),
    /// ];
    ///
    /// let mut graph = GraphAggregate::new()
    ///     .all_to_all(&source_nodes, &target_nodes)
    ///     .build();
    ///
    /// assert!(graph.is_valid());
    /// ```
    ///
    /// This will create a `Graph<T>` that looks like this:
    /// ``` text
    /// 0 -> [3, 4, 5]
    /// 1 -> [3, 4, 5]
    /// ```
    ///
    /// # Arguments
    /// * `one`: The first collection of `GraphNode`s.
    /// * `two`: The second collection of `GraphNode`s.
    pub fn all_to_all<G: AsRef<[GraphNode<T>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::AllToAll, one, two);
        self
    }

    /// Connects the `GraphNode`s in the first collection to the `GraphNode`s in the second collection
    /// in a one-to-one-self relationship. This means that each `GraphNode` in the first collection
    /// will be connected to the corresponding `GraphNode` in the second collection and then each `GraphNode`
    /// in the second collection will be connected to the corresponding `GraphNode` in the first collection.
    ///
    /// # Example
    /// ```
    /// use radiate::*;
    ///
    /// let source_nodes = vec![
    ///    GraphNode::new(0, NodeType::Vertex, 0),
    /// ];
    ///
    /// let target_nodes = vec![
    ///   GraphNode::new(0, NodeType::Edge, 0),
    /// ];
    ///
    /// let mut graph = GraphAggregate::new()
    ///   .one_to_self(&source_nodes, &target_nodes)
    ///   .build();
    ///
    /// assert!(graph.is_valid());
    /// ```
    /// This will create a `Graph<T>` that looks like this:
    /// ``` text
    /// 0 -> 1
    /// 1 -> 0
    /// ```
    ///
    /// # Arguments
    /// * `one`: The first collection of `GraphNode`s.
    /// * `two`: The second collection of `GraphNode`s.
    pub fn one_to_self<G: AsRef<[GraphNode<T>]>>(mut self, one: &'a G, two: &'a G) -> Self {
        self.connect(ConnectTypes::OneToSelf, one, two);
        self
    }

    /// Inserts a collection of `GraphNode`s into the `GraphAggregate<T>`.
    /// This method will take a collection of `GraphNode`s and insert them into the `GraphAggregate<T>` without connecting
    /// them to any other `GraphNode`s. Instead, it takes the relationships already represented within the `collection` and
    /// stores them for later use when connecting the `GraphNode`s together.
    /// This is useful for when you already have a `Graph` that is built and you want to add it to the `GraphAggregate<T>`.
    pub fn insert<G: AsRef<[GraphNode<T>]>>(mut self, collection: &'a G) -> Self {
        self.attach(collection.as_ref());
        self
    }
}

impl<'a, T: Clone> GraphAggregate<'a, T> {
    /// Connects the collections of `GraphNode`s together in a layered way meaning each `Graph` is connected
    /// `OneToOne` to the next `Graph` in the collection. This is useful for when you have a collection of `Graph`s
    /// that you want to connect together in a traditional layered way.
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

    /// Attaches a collection of `GraphNode`s to the `GraphAggregate<T>`. This method will take a collection of `GraphNode`s
    /// and add them and their relationships to the `GraphAggregate<T>`. When you provide a `Graph` to the `GraphAggregate<T>`,
    /// that already is internally connected, this method will take the relationships and store them for later use when connecting
    /// the `GraphNode`s and any other inputs provided to the `GraphAggregate<T>`.
    pub fn attach(&mut self, group: &'a [GraphNode<T>]) {
        for node in group.iter() {
            if !self.nodes.contains_key(&node.id()) {
                let node_id = &node.id();

                self.nodes.insert(node_id, node);
                self.node_order.insert(self.node_order.len(), node_id);

                for outgoing in group
                    .iter()
                    .filter(|item| node.outgoing().contains(&item.index()))
                {
                    self.relationships.push(Relationship {
                        source_id: node.id(),
                        target_id: outgoing.id(),
                    });
                }
            }
        }
    }
}

impl<'a, T: Clone> GraphAggregate<'a, T> {
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
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
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
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<T>>>();

        if !recurrent_outputs.is_empty() {
            return recurrent_outputs;
        }

        collection
            .as_ref()
            .iter()
            .enumerate()
            .filter(|(_, node)| node.incoming().is_empty())
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<T>>>()
    }

    fn get_inputs<G: AsRef<[GraphNode<T>]>>(&self, collection: &'a G) -> Vec<&'a GraphNode<T>> {
        let inputs = collection
            .as_ref()
            .iter()
            .enumerate()
            .take_while(|(_, node)| node.incoming().is_empty())
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
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
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<T>>>();

        if !recurrent_inputs.is_empty() {
            return recurrent_inputs;
        }

        collection
            .as_ref()
            .iter()
            .enumerate()
            .filter(|(_, node)| node.outgoing().is_empty())
            .map(|(idx, _)| collection.as_ref().get(idx).unwrap())
            .collect::<Vec<&GraphNode<T>>>()
    }
}
