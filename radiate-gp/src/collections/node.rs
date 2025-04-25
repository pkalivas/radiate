use crate::Arity;

/// [NodeType] is a soft identification for different nodes within a graph or tree structure.
///
/// Most of the time when reading a node's type, we can determine what kind of node it is
/// by the connections around it. For example, in a graph, if a node has 0 incoming connections,
/// it is likely an Input. Inversely, if a node has 0 outgoing connections, it is likely an Output. For
/// a tree, we can tell if it is a leaf or a vertex based on it's nunmber of children. However, when
/// building either a graph or tree, we usually want to specify which type of node we want to create.
/// Thus, we typically use this enum more for writing (building nodes) rather than reading (traversing nodes).
///
/// See the `GraphNode` and `TreeNode` implementations for more details or rules around how this is handled.
///
/// Because of this, the `NodeType` enum is a soft identification, and should be used as a hint rather than a strict rule.
/// The `node_type` method in the `Node` trait has guards around it within the `GraphNode` and `TreeNode` implementations
/// which handle this ambiguity and provide a more accurate node type when traversing the graph or tree. All that being
/// said, it is a very very rare case where the `node_type` method would return a different value than what is
/// specified in the `NodeType` enum - the only way this is possible is if the `NodeType` isn't supplied to
/// the node during creation.
///
/// Within each node (`GraphNode` or `TreeNode`), the [NodeType] is used to determine the validity of the
/// node given the value it holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// `Input` types are only used within graph structrues and are
    /// the starting point for data flow within the graph.
    Input,
    /// `Output` types are only used within graph structrues and are
    /// the endpoint for data flow within the graph.
    Output,
    /// `Vertex` types are used within both graph and tree structures
    /// to represent nodes that must have incoming (or parent) connections and
    /// outgoing connections (or children). This is a general purpose node type and
    /// is likely the type you think of when thinking of a generic node.
    Vertex,
    /// `Edge` types are only used within graph structures and represent
    /// nodes that have a single incoming connection and n outgoing connections.
    /// This is how we represent weights or other single-input nodes within a graph.
    Edge,
    /// `Leaf` types are used within tree structures to represent nodes
    /// that have no children. They are the endpoint (or output) of the tree structure.
    /// We use `Leaf` instead of output to avoid confusion and keep terminology consistent.
    Leaf,
    /// `Root` types are used within tree structures to represent the
    /// starting point of the tree. They are the first node in the tree structure and must
    /// have 0 parents.
    Root,
}

/// Node is a trait that abstracts out common information and behavior needed within the `GraphNode`
/// and `TreeNode` implementations. Both these nodes handle their connections differently, but they share
/// this common interface. Within this crate, we also handle these data structures a little differently
/// than they would usually be defined, so we leave the core implementation up to the struct, and allow
/// this trait to supply the 'radiate' interface for working with nodes.
pub trait Node {
    type Value;

    /// Get a reference to the node's value.
    fn value(&self) -> &Self::Value;

    /// Get a mutable reference to the node's value.
    fn value_mut(&mut self) -> &mut Self::Value;

    /// Get the [NodeType] of the node. As previously mentioned, if the [NodeType] is not supplied
    /// during creation, this value is determined by the node's relationship to the rest of
    /// the structure holding it. IE, a `GraphNode` with 0 incoming connections is likely an `Input`,
    /// while a `TreeNode` with 0 children is likely a `Leaf`.
    fn node_type(&self) -> NodeType;

    /// Get the arity of the node, which is the number of incoming connections it can have.
    /// In a genetic programming sense, this is the number of allowed inputs for a node.
    /// In a Graph, this is the number of allowed incoming connections while for a Tree,
    /// this is the number of children it is allowed to have.
    fn arity(&self) -> Arity;
}
