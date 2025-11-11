use crate::node::Node;
use crate::{Arity, NodeType};
use radiate_core::{Gene, Valid};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};

/// A unique identifier for nodes in a graph structure.
///
/// `GraphNodeId` is a newtype wrapper around a `u64` that provides a unique identifier
/// for each node in a graph. The ID is automatically generated using an atomic counter,
/// ensuring thread-safe unique ID generation across the application.
///
/// # Examples
/// ```
/// use radiate_gp::collections::GraphNodeId;
///
/// let id1 = GraphNodeId::new();
/// let id2 = GraphNodeId::new();
/// assert_ne!(id1, id2); // Each ID is unique
/// ```
///
/// # Implementation Details
/// * Uses an atomic counter (`AtomicU64`) to ensure thread-safe ID generation
/// * Implements `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `PartialOrd`, and `Ord`
/// * When the "serde" feature is enabled, implements `Serialize` and `Deserialize`
/// * Uses `#[repr(transparent)]` to ensure the same memory layout as `u64`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct GraphNodeId(u64);

impl GraphNodeId {
    pub fn new() -> Self {
        static GRAPH_NODE_ID: AtomicU64 = AtomicU64::new(0);
        GraphNodeId(GRAPH_NODE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

/// Represents the direction of connections in a graph node.
///
/// The [Direction] enum is used to specify whether a node's connections follow the
/// normal forward direction or create a backward (recurrent) connection. This is
/// particularly important for creating cyclic graphs and recurrent neural networks.
///
/// # Variants
/// * `Forward` - The default direction for normal graph connections. In a forward
///   connection, data flows from input nodes through intermediate nodes to output nodes.
/// * `Backward` - Indicates a recurrent connection where data can flow backwards,
///   creating cycles in the graph. This is used to implement recurrent neural networks
///   and other cyclic graph structures.
///
/// # Examples
/// ```
/// use radiate_gp::collections::{graphs::Direction, GraphNode, NodeType};
///
/// let mut node = GraphNode::new(0, NodeType::Vertex, 42);
/// assert_eq!(node.direction(), Direction::Forward);
///
/// // Create a recurrent connection
/// node.set_direction(Direction::Backward);
/// assert!(node.is_recurrent());
/// ```
///
/// # Usage in Graphs
/// * By default, graphs are directed acyclic graphs (DAGs) with all connections in the
///   `Forward` direction
/// * Setting a node's direction to `Backward` allows for cyclic connections
/// * The `Graph::set_cycles` method automatically sets appropriate nodes to `Backward`
///   direction when cycles are detected
///
/// # Implementation Details
/// * Implements `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, and `Hash`
/// * When the "serde" feature is enabled, implements `Serialize` and `Deserialize`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Direction {
    Forward,
    Backward,
}

/// A node in a graph structure that represents a single element with connections to other nodes.
///
/// The [GraphNode] struct is a fundamental building block for graph-based genetic programming in Radiate.
/// It represents a node in a directed graph that can have both incoming and outgoing connections to other nodes.
/// Each node has a unique identifier, an index in the graph, a value of type T, and maintains sets of incoming
/// and outgoing connections.
///
/// # Type Parameters
/// * `T` - The type of value stored in the node. This type must implement `Clone`, `PartialEq`, and other traits
///         required by the genetic programming operations.
///
/// # Fields
/// * `value` - The actual value stored in the node
/// * `id` - A unique identifier for the node ([GraphNodeId])
/// * `index` - The position of the node in the graph's node collection
/// * `direction` - The direction of the node's connections (Forward or Backward)
/// * `node_type` - Optional [NodeType] that specifies the role of the node (Input, Output, Vertex, Edge, etc.)
/// * `arity` - Optional [Arity] that specifies how many incoming connections the node can have. If
/// the arity is not supplied, the node will try it's best to determine it based on the node type and
/// the number of connections.
/// * `incoming` - Set of indices of nodes that have connections to this node
/// * `outgoing` - Set of indices of nodes that this node has connections to
///
/// # Examples
/// ```
/// use radiate_gp::{collections::{GraphNode, NodeType}, Arity};
///
/// // Create a new input node with value 42
/// let node = GraphNode::new(0, NodeType::Input, 42);
///
/// // Create a node with specific arity
/// // This node will be invalid if it has a number of incoming connections other than 2
/// let node_with_arity = GraphNode::with_arity(1, NodeType::Vertex, 42, Arity::Exact(2));
/// ```
///
/// # Node Types and Arity
/// The node's type and arity determine its behavior and validity:
/// * `Input` nodes should have no incoming connections and at least one outgoing connection
/// * `Output` nodes should have at least one incoming connection
/// * `Vertex` nodes can have both incoming and outgoing connections
/// * `Edge` nodes should have exactly one incoming and one outgoing connection
///
/// # Recurrent Connections
/// Nodes can form recurrent connections (cycles) in the graph by:
/// * Setting the node's direction to `Direction::Backward`
/// * Having a connection to itself (index in incoming/outgoing sets)
///
/// # Validity
/// A node is considered valid based on its type and connections:
/// * `Input` nodes are valid when they have no incoming connections and at least one outgoing connection
/// * `Output` nodes are valid when they have at least one incoming connection
/// * `Vertex` nodes are valid when they have both incoming and outgoing connections
/// * `Edge` nodes are valid when they have exactly one incoming and one outgoing connection
///
/// # Implementation Details
/// The struct implements several traits:
/// * `Node` - Provides common node behavior and access to value and type information
/// * `Gene` - Enables genetic operations for the node making it compatible with genetic algorithms
/// * `Valid` - Defines validity rules for the node
/// * `Debug` - Provides debug formatting
/// * `Clone`, `PartialEq` - Required for genetic programming operations
///
/// # Serialization
/// When the "serde" feature is enabled, the struct implements `Serialize` and `Deserialize` traits.
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphNode<T> {
    value: T,
    id: GraphNodeId,
    index: usize,
    direction: Direction,
    node_type: Option<NodeType>,
    arity: Option<Arity>,
    incoming: SmallVec<[usize; 8]>,
    outgoing: SmallVec<[usize; 8]>,
}

impl<T> GraphNode<T> {
    /// Creates a new [GraphNode] with the specified index, node type, and value.
    ///
    /// This is the most basic constructor for a graph node, initializing it with
    /// default direction (Forward) and no specific arity or node type.
    pub fn new(index: usize, node_type: NodeType, value: T) -> Self {
        GraphNode {
            id: GraphNodeId::new(),
            index,
            value,
            direction: Direction::Forward,
            node_type: Some(node_type),
            arity: None,
            incoming: SmallVec::new(),
            outgoing: SmallVec::new(),
        }
    }

    /// Creates a new [GraphNode] with the specified index, node type, value, and arity.
    ///
    /// This constructor allows for more control over the node's behavior by specifying
    /// the arity, which defines how many incoming connections the node can accept - if the
    /// number of connections does not match the arity, the node will be considered invalid.
    pub fn with_arity(index: usize, node_type: NodeType, value: T, arity: Arity) -> Self {
        GraphNode {
            id: GraphNodeId::new(),
            index,
            value,
            direction: Direction::Forward,
            node_type: Some(node_type),
            arity: Some(arity),
            incoming: SmallVec::new(),
            outgoing: SmallVec::new(),
        }
    }

    pub fn with_incoming<I: IntoIterator<Item = usize>>(mut self, incoming: I) -> Self {
        Self::set_sorted_unique(&mut self.incoming, incoming);
        self
    }

    pub fn with_outgoing<O: IntoIterator<Item = usize>>(mut self, outgoing: O) -> Self {
        Self::set_sorted_unique(&mut self.outgoing, outgoing);
        self
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn id(&self) -> &GraphNodeId {
        &self.id
    }

    pub fn is_recurrent(&self) -> bool {
        self.direction == Direction::Backward
            || self.incoming.contains(&self.index)
            || self.outgoing.contains(&self.index)
    }

    pub fn incoming(&self) -> &[usize] {
        &self.incoming
    }

    pub fn outgoing(&self) -> &[usize] {
        &self.outgoing
    }

    pub fn incoming_mut(&mut self) -> &mut [usize] {
        &mut self.incoming
    }

    pub fn outgoing_mut(&mut self) -> &mut [usize] {
        &mut self.outgoing
    }

    pub fn is_locked(&self) -> bool {
        match self.arity() {
            Arity::Any => false,
            _ => self.incoming.len() == *self.arity(),
        }
    }

    pub fn insert_incoming(&mut self, value: usize) {
        Self::insert_sorted_unique(&mut self.incoming, value);
    }

    pub fn remove_incoming(&mut self, value: &usize) {
        Self::remove_sorted(&mut self.incoming, value);
    }

    pub fn insert_outgoing(&mut self, value: usize) {
        Self::insert_sorted_unique(&mut self.outgoing, value);
    }

    pub fn remove_outgoing(&mut self, value: &usize) {
        Self::remove_sorted(&mut self.outgoing, value);
    }

    #[inline]
    fn insert_sorted_unique(v: &mut SmallVec<[usize; 8]>, value: usize) {
        match v.binary_search(&value) {
            Ok(_) => {}
            Err(pos) => v.insert(pos, value),
        }
    }

    #[inline]
    fn remove_sorted(v: &mut SmallVec<[usize; 8]>, value: &usize) {
        if let Ok(pos) = v.binary_search(value) {
            v.remove(pos);
        }
    }

    #[inline]
    fn set_sorted_unique(dst: &mut SmallVec<[usize; 8]>, src: impl IntoIterator<Item = usize>) {
        dst.clear();
        dst.extend(src);
        dst.sort_unstable();
        dst.dedup();
    }
}

/// Implementing the [Node] trait for [GraphNode]
/// This joins common functionality for nodes in a graph structure together.
impl<T> Node for GraphNode<T> {
    type Value = T;

    fn value(&self) -> &Self::Value {
        &self.value
    }

    fn value_mut(&mut self) -> &mut Self::Value {
        &mut self.value
    }

    fn node_type(&self) -> NodeType {
        if let Some(node_type) = self.node_type {
            return node_type;
        }

        let arity = self.arity();

        if let Arity::Any = arity {
            if self.outgoing.is_empty() && self.incoming.is_empty() {
                NodeType::Vertex
            } else if self.outgoing.is_empty() {
                NodeType::Output
            } else {
                NodeType::Vertex
            }
        } else if let Arity::Exact(1) = arity {
            if self.incoming.len() == 1 && self.outgoing.len() == 1 {
                NodeType::Edge
            } else {
                NodeType::Vertex
            }
        } else if let Arity::Zero = arity {
            NodeType::Input
        } else {
            NodeType::Vertex
        }
    }

    fn arity(&self) -> Arity {
        if let Some(node_type) = self.node_type {
            return self.arity.unwrap_or(match node_type {
                NodeType::Input => Arity::Zero,
                NodeType::Output => Arity::Any,
                NodeType::Vertex => Arity::Any,
                NodeType::Edge => Arity::Exact(1),
                NodeType::Leaf => Arity::Zero,
                NodeType::Root => Arity::Any,
            });
        }

        self.arity.unwrap_or(Arity::Any)
    }
}

impl<T> Gene for GraphNode<T>
where
    T: Clone + PartialEq,
{
    type Allele = T;

    fn allele(&self) -> &Self::Allele {
        self.value()
    }

    fn allele_mut(&mut self) -> &mut Self::Allele {
        &mut self.value
    }

    fn new_instance(&self) -> GraphNode<T> {
        GraphNode {
            id: GraphNodeId::new(),
            index: self.index,
            value: self.value.clone(),
            direction: self.direction,
            node_type: self.node_type,
            arity: self.arity,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> GraphNode<T> {
        GraphNode {
            id: GraphNodeId::new(),
            index: self.index,
            value: allele.clone(),
            direction: self.direction,
            node_type: self.node_type,
            arity: self.arity,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }
}

/// Implementing the [Valid] trait for [GraphNode]
/// This trait checks if the node is valid based on its type and connections.
/// A valid node must have the correct number of incoming and outgoing connections
/// according to its arity and node type.
///
/// A node is considered valid based on its type and connections:
/// * `Input` nodes are valid when they have no incoming connections and at least one outgoing connection
/// * `Output` nodes are valid when they have at least one incoming connection
/// * `Vertex` nodes are valid when they have both incoming and outgoing connections
/// * `Edge` nodes are valid when they have exactly one incoming and one outgoing connection
impl<T> Valid for GraphNode<T> {
    #[inline]
    fn is_valid(&self) -> bool {
        match self.node_type() {
            NodeType::Input => self.incoming.is_empty() && !self.outgoing.is_empty(),
            NodeType::Output => {
                (!self.incoming.is_empty())
                    && (self.incoming.len() == *self.arity() || self.arity() == Arity::Any)
            }
            NodeType::Vertex => {
                if !self.incoming.is_empty() && !self.outgoing.is_empty() {
                    if let Arity::Exact(n) = self.arity() {
                        return self.incoming.len() == n;
                    } else if self.arity() == Arity::Any {
                        return true;
                    }
                }
                false
            }
            NodeType::Edge => {
                if self.arity() == Arity::Exact(1) {
                    return self.incoming.len() == 1 && self.outgoing.len() == 1;
                }

                false
            }
            _ => false,
        }
    }
}

impl<T> From<(usize, NodeType, T)> for GraphNode<T> {
    fn from((index, node_type, value): (usize, NodeType, T)) -> Self {
        GraphNode::new(index, node_type, value)
    }
}

impl<T: Default> From<(usize, T)> for GraphNode<T> {
    fn from((index, value): (usize, T)) -> Self {
        GraphNode {
            index,
            id: GraphNodeId::new(),
            value,
            direction: Direction::Forward,
            node_type: None,
            arity: None,
            incoming: SmallVec::new(),
            outgoing: SmallVec::new(),
        }
    }
}

impl<T> From<(usize, NodeType, T, Arity)> for GraphNode<T> {
    fn from((index, node_type, value, arity): (usize, NodeType, T, Arity)) -> Self {
        GraphNode::with_arity(index, node_type, value, arity)
    }
}

impl<T: Default> From<(usize, T, Arity)> for GraphNode<T> {
    fn from((index, value, arity): (usize, T, Arity)) -> Self {
        GraphNode {
            index,
            id: GraphNodeId::new(),
            value,
            direction: Direction::Forward,
            node_type: None,
            arity: Some(arity),
            incoming: SmallVec::new(),
            outgoing: SmallVec::new(),
        }
    }
}

impl<T, I> From<(usize, NodeType, T, I, I)> for GraphNode<T>
where
    I: IntoIterator<Item = usize>,
{
    fn from((index, node_type, value, incoming, outgoing): (usize, NodeType, T, I, I)) -> Self {
        let mut incoming_indices = SmallVec::<[usize; 8]>::new();
        let mut outgoing_indices = SmallVec::<[usize; 8]>::new();

        GraphNode::<T>::set_sorted_unique(&mut incoming_indices, incoming);
        GraphNode::<T>::set_sorted_unique(&mut outgoing_indices, outgoing);

        GraphNode {
            index,
            id: GraphNodeId::new(),
            value,
            direction: Direction::Forward,
            node_type: Some(node_type),
            arity: None,
            incoming: incoming_indices,
            outgoing: outgoing_indices,
        }
    }
}

impl<T: Default> Default for GraphNode<T> {
    fn default() -> Self {
        GraphNode {
            id: GraphNodeId::new(),
            index: 0,
            value: Default::default(),
            direction: Direction::Forward,
            node_type: None,
            arity: None,
            incoming: SmallVec::new(),
            outgoing: SmallVec::new(),
        }
    }
}

impl<T: Debug> Debug for GraphNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let incoming = self
            .incoming
            .iter()
            .map(|idx| idx.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        write!(
            f,
            "[{:<3}] [{:<7?}] {:>10?} :: {:<10} {:<12} V:{:<5} R:{:<5} {:<2} {:<2} < [{}]",
            self.index,
            self.id.0,
            format!("{:?}", self.node_type())[..3].to_owned(),
            self.arity(),
            format!("{:?}", self.value).to_owned(),
            self.is_valid(),
            self.is_recurrent(),
            self.incoming.len(),
            self.outgoing.len(),
            incoming,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeType;

    #[test]
    fn test_graph_node_default() {
        let node = GraphNode::<usize>::default();

        assert_eq!(node.index(), 0);
        assert_eq!(node.node_type(), NodeType::Vertex);
        assert_eq!(node.arity(), Arity::Any);
        assert!(!node.is_valid());
        assert!(!node.is_recurrent());
        assert_eq!(node.incoming(), &[] as &[usize]);
        assert_eq!(node.outgoing(), &[] as &[usize]);
    }

    #[test]
    fn test_graph_node() {
        let node = GraphNode::new(0, NodeType::Input, 0.0);

        assert_eq!(node.index(), 0);
        assert_eq!(node.node_type(), NodeType::Input);
        assert_eq!(node.arity(), Arity::Zero);
        assert!(!node.is_valid());
        assert!(!node.is_recurrent());
        assert_eq!(node.incoming(), &[] as &[usize]);
        assert_eq!(node.outgoing(), &[] as &[usize]);
    }

    #[test]
    fn test_graph_node_with_arity() {
        let node = GraphNode::with_arity(0, NodeType::Input, 0.0, Arity::Zero);

        assert_eq!(node.index(), 0);
        assert_eq!(node.node_type(), NodeType::Input);
        assert_eq!(node.arity(), Arity::Zero);
        assert!(!node.is_valid());
        assert!(!node.is_recurrent());
        assert_eq!(node.incoming(), &[] as &[usize]);
        assert_eq!(node.outgoing(), &[] as &[usize]);
    }

    #[test]
    fn test_graph_node_with_allele() {
        let node = GraphNode::new(0, NodeType::Input, 0.0);

        let new_node = node.with_allele(&1.0);
        assert_eq!(new_node.index(), 0);
        assert_eq!(new_node.node_type(), NodeType::Input);
        assert_eq!(new_node.arity(), Arity::Zero);
        assert!(!new_node.is_valid());
        assert!(!new_node.is_recurrent());
        assert_eq!(new_node.incoming(), &[] as &[usize]);
        assert_eq!(new_node.outgoing(), &[] as &[usize]);
    }

    #[test]
    fn test_graph_node_with_direction() {
        let mut node_one = GraphNode::new(0, NodeType::Input, 0.0);

        assert!(!node_one.is_recurrent());
        node_one.set_direction(Direction::Backward);
        assert!(node_one.is_recurrent());

        let mut node_two = GraphNode::new(0, NodeType::Input, 0.0);

        assert!(!node_two.is_recurrent());
        node_two.insert_incoming(0);
        assert!(node_two.is_recurrent());
    }

    #[test]
    fn graph_node_from_fns_produce_valid_arities() {
        let node = GraphNode::from((0, NodeType::Input, 0.0));
        assert_eq!(node.arity(), Arity::Zero);

        let node = GraphNode::from((0, NodeType::Output, 0.0));
        assert_eq!(node.arity(), Arity::Any);

        let node = GraphNode::from((0, NodeType::Vertex, 0.0));
        assert_eq!(node.arity(), Arity::Any);

        let node = GraphNode::from((0, NodeType::Edge, 0.0));
        assert_eq!(node.arity(), Arity::Exact(1));

        let node = GraphNode::from((0, NodeType::Input, 0.0, Arity::Zero));
        assert_eq!(node.arity(), Arity::Zero);

        let node = GraphNode::from((0, NodeType::Output, 0.0, Arity::Any));
        assert_eq!(node.arity(), Arity::Any);

        let node = GraphNode::from((0, NodeType::Vertex, 0.0, Arity::Any));
        assert_eq!(node.arity(), Arity::Any);

        let node = GraphNode::from((0, NodeType::Edge, 0.0, Arity::Exact(1)));
        assert_eq!(node.arity(), Arity::Exact(1));
    }

    #[test]
    fn test_graph_node_validity() {
        let mut input_node = GraphNode::new(0, NodeType::Input, 0.0);
        assert!(!input_node.is_valid());

        input_node.insert_outgoing(1);
        assert!(input_node.is_valid());

        let mut output_node = GraphNode::new(1, NodeType::Output, 0.0);
        assert!(!output_node.is_valid());

        output_node.insert_incoming(0);
        assert!(output_node.is_valid());
    }

    #[test]
    fn test_graph_node_connections_sorted() {
        let mut node = GraphNode::new(0, NodeType::Vertex, 0.0);

        node.insert_incoming(3);
        node.insert_incoming(1);
        node.insert_incoming(2);
        node.insert_incoming(2); // Duplicate

        assert_eq!(node.incoming(), &[1, 2, 3]);

        node.insert_outgoing(5);
        node.insert_outgoing(4);
        node.insert_outgoing(6);
        node.insert_outgoing(5); // Duplicate

        assert_eq!(node.outgoing(), &[4, 5, 6]);

        node.remove_incoming(&2);
        assert_eq!(node.incoming(), &[1, 3]);

        node.remove_outgoing(&5);
        assert_eq!(node.outgoing(), &[4, 6]);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_graph_node_serde() {
        let node = GraphNode::new(0, NodeType::Input, 42.0);
        let serialized = serde_json::to_string(&node).unwrap();
        let deserialized = serde_json::from_str::<GraphNode<f32>>(&serialized).unwrap();

        assert_eq!(node, deserialized);
        assert_eq!(node.value(), &42.0);
        assert_eq!(deserialized.value(), &42.0);
    }
}
