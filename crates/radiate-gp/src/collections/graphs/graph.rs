use super::transaction::TransactionResult;
use crate::collections::graphs::GraphTransaction;
use crate::collections::{Direction, GraphNode};
use crate::{Node, NodeType};
use radiate_core::Valid;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

/// A graph structure that represents a collection of interconnected nodes.
///
/// The [Graph] struct is a fundamental data structure in Radiate's genetic programming system.
/// Unlike traditional graphs that separate edges and vertices, this graph is a collection of nodes
/// where each node maintains its own connections. Each node has a unique index that corresponds
/// to its position in the internal vector, and connections are represented by these indices.
///
/// # Type Parameters
/// * `T` - The type of value stored in each node. This type must implement `Clone`, `PartialEq`,
///         and other traits required by the genetic programming operations.
///
/// # Structure
/// A [Graph] is simply a 'Vec' of [GraphNode]'s.
///
/// It's important to note that this graph differs from a traditional graph in that it is not
/// a collection of edges and vertices. Instead, it is a collection of nodes that are connected
/// to one another. Each node has a unique index that is used to reference it in the graph
/// and must be identical to its position in the 'Vec'.
/// Each [GraphNode] has a set of ordered incoming and outgoing connections. These connections are
/// represented by the index of the connected node in the graph. Because of this representation,
/// an edge is not a separate entity, it's just a node. The 'NodeType' enum is used to distinguish
/// different types of nodes. This allows for a more flexible representation of the graph
/// while still maintaining the ability to represent traditional graphs.
///
/// By default, a [Graph] is a directed acyclic graph (DAG). However, it is possible to create
/// cycles in the graph by setting the 'direction' field of a [GraphNode] to [Direction::Backward].
/// The [Graph] struct provides methods for attaching and detaching nodes from one another.
/// It also provides methods for iterating over the nodes in the graph in a pseudo-topological order.
///
/// Each node:
/// * Has a unique index matching its position in the vector
/// * Maintains sets of incoming and outgoing connections
/// * Can be of different types (Input, Output, Vertex, Edge) as defined by `NodeType`
/// * Can have a direction (Forward or Backward) for handling cycles
///
/// # Examples
/// ```
/// use radiate_gp::{Graph, NodeType, Op};
///
/// // Create a simple graph with one input and one output node
/// let mut graph = Graph::default();
/// let input_idx = graph.insert(NodeType::Input, Op::var(0));
/// let output_idx = graph.insert(NodeType::Output, Op::linear());
/// graph.attach(input_idx, output_idx);
///
/// // Create a directed graph with 2 inputs and 2 outputs
/// let values = vec![
///     (NodeType::Input, vec![Op::var(0), Op::var(1)]),
///     (NodeType::Output, vec![Op::sigmoid(), Op::tanh()]),
/// ];
/// let graph = Graph::directed(2, 2, values);
/// ```
///
/// # Graph Types
/// The struct provides several factory methods for creating different types of graphs:
/// * `directed()` - Creates a directed acyclic graph (DAG) with specified input and output nodes
/// * `recurrent()` - Creates a graph with recurrent connections
/// * `weighted_directed()` - Creates a directed graph with weighted edges
/// * `weighted_recurrent()` - Creates a recurrent graph with weighted edges
///
/// # Node Operations
/// The struct provides methods for manipulating nodes:
/// * `insert()` - Adds a new node and returns its index
/// * `push()` - Adds a node to the end of the graph
/// * `pop()` - Removes and returns the last node
/// * `get()` - Returns a reference to a node by index
/// * `get_mut()` - Returns a mutable reference to a node by index
///
/// # Connection Management
/// The struct provides methods for managing node connections:
/// * `attach()` - Creates a connection between two nodes
/// * `detach()` - Removes a connection between two nodes
/// * `set_cycles()` - Configures nodes to support cyclic connections
///
/// # Graph Traversal
/// The struct implements the `GraphIterator` trait, providing:
/// * `iter_topological()` - Traverses the graph in a pseudo-topological order
/// * `iter()` - Iterates over all nodes
/// * `iter_mut()` - Iterates over all nodes with mutable references
///
/// # Node Type Queries
/// The struct provides methods to get nodes by type:
/// * `inputs()` - Returns all input nodes
/// * `outputs()` - Returns all output nodes
/// * `vertices()` - Returns all vertex nodes
/// * `edges()` - Returns all edge nodes
///
/// # Graph Properties
/// The struct provides methods to query graph properties:
/// * `len()` - Returns the number of nodes
/// * `is_empty()` - Checks if the graph is empty
/// * `is_valid()` - Checks if all nodes in the graph are valid
///
/// # Implementation Details
/// The struct implements several traits:
/// * `Clone` - Allows cloning of the entire graph
/// * `PartialEq` - Enables equality comparison between graphs
/// * `Default` - Provides a way to create an empty graph
/// * `Debug` - Provides debug formatting
/// * `AsRef<[GraphNode<T>]>` - Allows treating the graph as a slice of nodes
/// * `AsMut<[GraphNode<T>]>` - Allows treating the graph as a mutable slice of nodes
/// * `Index<usize>` - Enables indexing with `graph[index]`
/// * `IndexMut<usize>` - Enables mutable indexing with `graph[index]`
/// * `IntoIterator` - Allows iterating over nodes
/// * `FromIterator<GraphNode<T>>` - Allows creating a graph from an iterator of nodes
///
/// # Genetic Programming
/// The [Graph] struct is particularly useful in genetic programming as it can represent:
/// * Neural networks (using `Op<f32>` or `Op<bool>` for values)
/// * Decision graphs
/// * Program flow graphs
/// * Other interconnected structures
///
/// It supports genetic operations through the `GraphChromosome` type, including:
///
/// # Serialization
/// When the "serde" feature is enabled, the struct implements `Serialize` and `Deserialize` traits.
///
/// # Performance Considerations
/// * All nodes (vertices, edges, inputs, outputs) are represented as [GraphNode] instances
/// * Node lookups are O(1) due to vector indexing
/// * Connection operations (attach/detach) are O(log n) due to BTreeSet usage for incoming/outgoing connections
/// * Graph traversal is O(V + E) where V is the number of nodes and E is the total number of connections
/// * Memory usage is O(V + E) for storing nodes and their connections
/// * The distinction between node types (Vertex, Edge, Input, Output) is purely semantic and
///   does not affect the underlying data structure or performance characteristics
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Graph<T> {
    nodes: Vec<GraphNode<T>>,
}

impl<T> Graph<T> {
    /// Create a new 'Graph' from a 'Vec' of [GraphNode]s.
    ///
    /// # Arguments
    /// - nodes: A 'Vec' of [GraphNode]s.
    pub fn new(nodes: Vec<GraphNode<T>>) -> Self {
        Graph { nodes }
    }

    pub fn take_nodes(&mut self) -> Vec<GraphNode<T>> {
        std::mem::take(&mut self.nodes)
    }

    pub fn push(&mut self, node: impl Into<GraphNode<T>>) {
        self.nodes.push(node.into());
    }

    pub fn insert(&mut self, node_type: NodeType, val: T) -> usize {
        self.push((self.len(), node_type, val));
        self.len() - 1
    }

    pub fn pop(&mut self) -> Option<GraphNode<T>> {
        self.nodes.pop()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut GraphNode<T>> {
        self.nodes.get_mut(index)
    }

    pub fn get(&self, index: usize) -> Option<&GraphNode<T>> {
        self.nodes.get(index)
    }

    pub fn iter(&self) -> impl Iterator<Item = &GraphNode<T>> {
        self.nodes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut GraphNode<T>> {
        self.nodes.iter_mut()
    }

    pub fn inputs(&self) -> impl Iterator<Item = &GraphNode<T>> {
        self.get_nodes_of_type(NodeType::Input)
    }

    pub fn outputs(&self) -> impl Iterator<Item = &GraphNode<T>> {
        self.get_nodes_of_type(NodeType::Output)
    }

    pub fn vertices(&self) -> impl Iterator<Item = &GraphNode<T>> {
        self.get_nodes_of_type(NodeType::Vertex)
    }

    pub fn edges(&self) -> impl Iterator<Item = &GraphNode<T>> {
        self.get_nodes_of_type(NodeType::Edge)
    }

    /// Attach and detach nodes from one another. This is the primary way to modify the graph.
    /// Note that this method does not check if the nodes are already connected. This is because
    /// the connections are represented by 'BTreeSet's which do not allow duplicates.
    /// Its also important to note that the 'incoming' and 'outgoing' indices are the indices of the
    /// nodes in the graph, not the indices of the connections in the 'incoming' and 'outgoing' 'BTreeSet's.
    /// We must also remember that the [GraphNode] cares about the 'Arity' of the 'Operation' it contains,
    /// so if we add a connection that would violate the 'Arity' of the 'Operation', the connection will result
    /// in a [GraphNode] that is not 'Valid'.
    ///
    /// Attaches the node at the 'incoming' index to the node at the 'outgoing' index.
    /// This means that the node at the 'incoming' index will have an outgoing connection
    /// to the node at the 'outgoing' index, and the node at the 'outgoing' index will have
    /// an incoming connection from the node at the 'incoming' index.
    ///
    /// # Arguments
    /// - incoming: The index of the node that will have an outgoing connection to the node at the 'outgoing' index.
    /// - outgoing: The index of the node that will have an incoming connection from the node at the 'incoming' index.
    pub fn attach(&mut self, incoming: usize, outgoing: usize) -> &mut Self {
        self.as_mut()[incoming].insert_outgoing(outgoing);
        self.as_mut()[outgoing].insert_incoming(incoming);
        self
    }
    /// Detaches the node at the 'incoming' index from the node at the 'outgoing' index.
    /// This means that the node at the 'incoming' index will no longer have an outgoing connection
    /// to the node at the 'outgoing' index, and the node at the 'outgoing' index will no longer have
    /// an incoming connection from the node at the 'incoming' index.
    ///
    /// # Arguments
    /// - incoming: The index of the node that will no longer have an outgoing connection to the node at the 'outgoing' index.
    /// - outgoing: The index of the node that will no longer have an incoming connection from the node at the 'incoming' index.
    pub fn detach(&mut self, incoming: usize, outgoing: usize) -> &mut Self {
        self.as_mut()[incoming].remove_outgoing(&outgoing);
        self.as_mut()[outgoing].remove_incoming(&incoming);
        self
    }

    /// tries to modify the graph using a [GraphTransaction]. If the transaction is successful,
    /// we return true and do nothing. If the transaction is not successful, we roll back the transaction
    /// by undoing all the changes made by the transaction and return false.
    ///
    /// # Arguments
    ///  - mutation: A closure that takes a mutable reference to a [GraphTransaction] and returns a 'bool'.
    #[inline]
    pub fn try_modify<F>(&mut self, mutation: F) -> TransactionResult<T>
    where
        F: FnOnce(GraphTransaction<T>) -> TransactionResult<T>,
        T: Clone,
    {
        mutation(GraphTransaction::new(self))
    }

    /// Given a list of node indices, this function will set the 'direction' field of the nodes
    /// at those indices to [Direction::Backward] if they are part of a cycle. If they are not part
    /// of a cycle, the 'direction' field will be set to [Direction::Forward].
    /// If no indices are provided, the function will set the 'direction' field of all nodes in the graph.
    #[inline]
    pub fn set_cycles(&mut self, indecies: Vec<usize>) {
        if indecies.is_empty() {
            let all_indices = self
                .as_ref()
                .iter()
                .map(|node| node.index())
                .collect::<Vec<usize>>();

            return self.set_cycles(all_indices);
        }

        for idx in indecies {
            let cycles = self.get_cycles(idx);

            if cycles.is_empty() {
                if let Some(node) = self.get_mut(idx) {
                    node.set_direction(Direction::Forward);
                }
            } else {
                for cycle in cycles {
                    if let Some(node) = self.get_mut(cycle) {
                        node.set_direction(Direction::Backward);
                    }
                }
            }
        }
    }

    /// Get the cycles in the graph that include the node at the specified index.
    ///
    /// # Arguments
    /// - index: The index of the node to get the cycles for.
    #[inline]
    pub fn get_cycles(&self, from: usize) -> std::collections::HashSet<usize> {
        let n = self.len();
        let mut on_stack = vec![false; n];
        let mut visited = vec![false; n];
        let mut cycles = vec![false; n];
        let mut stack: Vec<usize> = Vec::with_capacity(n.min(64));

        fn dfs<T>(
            g: &Graph<T>,
            u: usize,
            visited: &mut [bool],
            on_stack: &mut [bool],
            cycles: &mut [bool],
            stack: &mut Vec<usize>,
        ) {
            visited[u] = true;
            on_stack[u] = true;
            stack.push(u);

            for &v in g.get(u).unwrap().outgoing() {
                if !visited[v] {
                    dfs(g, v, visited, on_stack, cycles, stack);
                } else if on_stack[v] {
                    let start = stack.iter().rposition(|&x| x == v).unwrap();
                    for &w in &stack[start..] {
                        cycles[w] = true;
                    }
                }
            }

            stack.pop();
            on_stack[u] = false;
        }

        dfs(
            self,
            from,
            &mut visited,
            &mut on_stack,
            &mut cycles,
            &mut stack,
        );

        // collect results
        let mut out = HashSet::with_capacity(stack.len());
        for (i, &c) in cycles.iter().enumerate() {
            if c {
                out.insert(i);
            }
        }
        out
    }

    #[inline]
    fn get_nodes_of_type(&self, node_type: NodeType) -> impl Iterator<Item = &GraphNode<T>> {
        self.nodes
            .iter()
            .filter(move |node| node.node_type() == node_type)
    }
}

impl<T> Valid for Graph<T> {
    #[inline]
    fn is_valid(&self) -> bool {
        self.iter().all(|node| node.is_valid())
    }
}

impl<T> AsRef<[GraphNode<T>]> for Graph<T> {
    fn as_ref(&self) -> &[GraphNode<T>] {
        &self.nodes
    }
}

impl<T> AsMut<[GraphNode<T>]> for Graph<T> {
    fn as_mut(&mut self) -> &mut [GraphNode<T>] {
        &mut self.nodes
    }
}

impl<T> Index<usize> for Graph<T> {
    type Output = GraphNode<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<T> IndexMut<usize> for Graph<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

impl<T> IntoIterator for Graph<T> {
    type Item = GraphNode<T>;
    type IntoIter = std::vec::IntoIter<GraphNode<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<T> FromIterator<GraphNode<T>> for Graph<T> {
    fn from_iter<I: IntoIterator<Item = GraphNode<T>>>(iter: I) -> Self {
        Graph {
            nodes: iter.into_iter().collect(),
        }
    }
}

impl<T> Default for Graph<T> {
    fn default() -> Self {
        Graph { nodes: Vec::new() }
    }
}

impl<T: Debug> Debug for Graph<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Graph {{\n")?;
        for node in self.as_ref() {
            write!(f, "  {:?},\n", node)?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Arity, Node, Op};

    #[test]
    fn test_graph_is_valid() {
        let mut graph_one = Graph::default();
        graph_one.push((0, NodeType::Input, 123));
        graph_one.push((1, NodeType::Output, 42));
        graph_one.attach(0, 1);

        let mut graph_two = Graph::default();
        graph_two.push((0, NodeType::Input, 0));
        graph_two.push((1, NodeType::Vertex, 1));

        assert!(graph_one.is_valid());
        assert!(!graph_two.is_valid());
    }

    #[test]
    fn test_graph_attach() {
        let mut graph = Graph::default();
        graph.push((0, NodeType::Input, 0));
        graph.push((1, NodeType::Output, 1));
        graph.attach(0, 1);

        assert_eq!(graph[0].outgoing(), &[1]);
        assert_eq!(graph[1].incoming(), &[0]);
    }

    #[test]
    fn test_graph_node_creations() {
        let mut graph_one = Graph::from_iter(vec![
            GraphNode::new(0, NodeType::Input, 0),
            GraphNode::new(1, NodeType::Vertex, 1),
            GraphNode::new(2, NodeType::Output, 1),
        ]);

        graph_one.attach(0, 1).attach(1, 2);

        assert_eq!(graph_one.len(), 3);
        assert!(graph_one.is_valid());
        assert_eq!(graph_one[0].arity(), Arity::Zero);
        assert_eq!(graph_one[1].arity(), Arity::Any);
        assert_eq!(graph_one[2].arity(), Arity::Any);

        let mut graph_two = Graph::new(vec![
            GraphNode::new(0, NodeType::Input, Op::var(0)),
            GraphNode::new(1, NodeType::Input, Op::constant(5.0)),
            GraphNode::with_arity(2, NodeType::Vertex, Op::add(), Arity::Exact(2)),
            GraphNode::new(3, NodeType::Output, Op::linear()),
        ]);

        graph_two.attach(0, 2).attach(1, 2).attach(2, 3);

        assert_eq!(graph_two.len(), 4);
        assert!(graph_two.is_valid());
        assert_eq!(graph_two[0].arity(), Arity::Zero);
        assert_eq!(graph_two[1].arity(), Arity::Zero);
        assert_eq!(graph_two[2].arity(), Arity::Exact(2));
        assert_eq!(graph_two[3].arity(), Arity::Any);
    }

    #[test]
    fn test_simple_graph() {
        let mut graph = Graph::<i32>::default();

        let idx_one = graph.insert(NodeType::Input, 0);
        let idx_two = graph.insert(NodeType::Vertex, 1);
        let idx_three = graph.insert(NodeType::Output, 2);

        graph.attach(idx_one, idx_two).attach(idx_two, idx_three);

        assert_eq!(graph.len(), 3);

        assert!(graph.is_valid());
        assert!(graph[0].is_valid());
        assert!(graph[1].is_valid());
        assert!(graph[2].is_valid());

        assert_eq!(graph[0].incoming().len(), 0);
        assert_eq!(graph[0].outgoing().len(), 1);
        assert_eq!(graph[1].incoming().len(), 1);
        assert_eq!(graph[1].outgoing().len(), 1);
        assert_eq!(graph[2].incoming().len(), 1);
        assert_eq!(graph[2].outgoing().len(), 0);
    }

    #[test]
    fn test_graph_with_cycles() {
        let mut graph = Graph::<i32>::default();

        graph.insert(NodeType::Input, 0);
        graph.insert(NodeType::Vertex, 1);
        graph.insert(NodeType::Vertex, 2);
        graph.insert(NodeType::Output, 3);

        graph.attach(0, 1).attach(1, 2).attach(2, 1).attach(2, 3);

        assert_eq!(graph.len(), 4);

        assert!(graph.is_valid());
        assert!(graph[0].is_valid());
        assert!(graph[1].is_valid());
        assert!(graph[2].is_valid());
        assert!(graph[3].is_valid());

        assert_eq!(graph[0].incoming().len(), 0);
        assert_eq!(graph[0].outgoing().len(), 1);
        assert_eq!(graph[1].incoming().len(), 2);
        assert_eq!(graph[1].outgoing().len(), 1);
        assert_eq!(graph[2].incoming().len(), 1);
        assert_eq!(graph[2].outgoing().len(), 2);
        assert_eq!(graph[3].incoming().len(), 1);
        assert_eq!(graph[3].outgoing().len(), 0);
    }

    #[test]
    fn test_graph_with_cycles_and_recurrent_nodes() {
        let mut graph = Graph::<i32>::default();

        let idx_one = graph.insert(NodeType::Input, 0);
        let idx_two = graph.insert(NodeType::Vertex, 1);
        let idx_three = graph.insert(NodeType::Vertex, 2);
        let idx_four = graph.insert(NodeType::Output, 3);

        graph
            .attach(idx_one, idx_two)
            .attach(idx_two, idx_three)
            .attach(idx_three, idx_two)
            .attach(idx_three, idx_four)
            .attach(idx_four, idx_two);

        graph.set_cycles(vec![]);

        assert_eq!(graph.len(), 4);

        assert!(graph.is_valid());
        assert!(graph[0].is_valid());
        assert!(graph[1].is_valid());
        assert!(graph[2].is_valid());
        assert!(graph[3].is_valid());

        assert_eq!(graph[0].incoming().len(), 0);
        assert_eq!(graph[0].outgoing().len(), 1);
        assert_eq!(graph[1].incoming().len(), 3);
        assert_eq!(graph[1].outgoing().len(), 1);
        assert_eq!(graph[2].incoming().len(), 1);
        assert_eq!(graph[2].outgoing().len(), 2);
        assert_eq!(graph[3].incoming().len(), 1);
        assert_eq!(graph[3].outgoing().len(), 1);

        assert_eq!(graph[0].direction(), Direction::Forward);
        assert_eq!(graph[1].direction(), Direction::Backward);
        assert_eq!(graph[2].direction(), Direction::Backward);
        assert_eq!(graph[3].direction(), Direction::Backward);
    }

    #[test]
    fn test_graph_clone_and_partial_eq() {
        let mut graph1 = Graph::default();
        let input_idx = graph1.insert(NodeType::Input, 42);
        let output_idx = graph1.insert(NodeType::Output, 24);
        graph1.attach(input_idx, output_idx);

        let graph2 = graph1.clone();
        assert_eq!(graph1, graph2);

        let mut graph3 = graph1.clone();
        graph3[input_idx].set_direction(Direction::Backward);
        assert_ne!(graph1, graph3);

        let mut graph4 = graph1.clone();
        if let Some(node) = graph4.get_mut(input_idx) {
            *node.value_mut() = 100;
        }
        assert_ne!(graph1, graph4);
    }

    #[test]
    fn test_graph_arity_validation() {
        let mut graph = Graph::default();
        let input_idx = graph.insert(NodeType::Input, 0);
        graph.push((1, NodeType::Vertex, 1, Arity::Exact(2)));
        let output_idx = graph.insert(NodeType::Output, 2);

        graph.attach(input_idx, 1);
        graph.attach(1, output_idx);

        // Should be invalid - vertex needs exactly 2 incoming connections
        assert!(!graph.is_valid());

        // Add one connection - should still be invalid, connections are unique so this just
        // replaces the existing one with the same index
        graph.attach(input_idx, 1);
        assert!(!graph.is_valid());

        // Add third connection - should still be valid with Arity::Any
        let input3_idx = graph.insert(NodeType::Input, 3);
        graph.attach(input3_idx, 1);
        println!("{:?}", graph);
        assert!(graph.is_valid());
    }

    #[test]
    fn test_graph_indexing() {
        let mut graph = Graph::default();
        let input_idx = graph.insert(NodeType::Input, 42);
        let output_idx = graph.insert(NodeType::Output, 24);

        // Test Index trait
        assert_eq!(graph[input_idx].value(), &42);
        assert_eq!(graph[output_idx].value(), &24);

        // Test IndexMut trait
        graph[input_idx].set_direction(Direction::Backward);
        assert_eq!(graph[input_idx].direction(), Direction::Backward);

        // Test get() and get_mut()
        assert_eq!(graph.get(input_idx).unwrap().value(), &42);
        assert_eq!(graph.get_mut(output_idx).unwrap().value(), &24);

        // Test out of bounds
        assert!(graph.get(999).is_none());
        assert!(graph.get_mut(999).is_none());
    }

    #[test]
    fn test_graph_node_type_queries() {
        let mut graph = Graph::default();
        graph.insert(NodeType::Input, 0);
        graph.insert(NodeType::Input, 1);
        graph.insert(NodeType::Vertex, 2);
        graph.insert(NodeType::Vertex, 3);
        graph.insert(NodeType::Output, 4);
        graph.insert(NodeType::Output, 5);

        // Test inputs()
        let inputs = graph.inputs().collect::<Vec<_>>();
        assert_eq!(inputs.len(), 2);
        assert!(
            inputs
                .iter()
                .all(|node| node.node_type() == NodeType::Input)
        );

        // Test vertices()
        let vertices = graph.vertices().collect::<Vec<_>>();
        assert_eq!(vertices.len(), 2);
        assert!(
            vertices
                .iter()
                .all(|node| node.node_type() == NodeType::Vertex)
        );

        // Test outputs()
        let outputs = graph.outputs().collect::<Vec<_>>();
        assert_eq!(outputs.len(), 2);
        assert!(
            outputs
                .iter()
                .all(|node| node.node_type() == NodeType::Output)
        );
    }

    #[test]
    fn test_graph_iterators() {
        let mut graph = Graph::default();
        let input_idx = graph.insert(NodeType::Input, 0);
        let vertex_idx = graph.insert(NodeType::Vertex, 1);
        let output_idx = graph.insert(NodeType::Output, 2);

        graph.attach(input_idx, vertex_idx);
        graph.attach(vertex_idx, output_idx);

        // Test iter()
        let nodes: Vec<_> = graph.iter().collect();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].value(), &0);
        assert_eq!(nodes[1].value(), &1);
        assert_eq!(nodes[2].value(), &2);

        // Test iter_mut()
        for node in graph.iter_mut() {
            if node.node_type() == NodeType::Vertex {
                node.set_direction(Direction::Backward);
            }
        }
        assert_eq!(graph[vertex_idx].direction(), Direction::Backward);

        // Test into_iter()
        let values: Vec<_> = graph.into_iter().map(|node| *node.value()).collect();
        assert_eq!(values, vec![0, 1, 2]);
    }

    #[test]
    fn test_graph_detach() {
        let mut graph = Graph::default();
        let input_idx = graph.insert(NodeType::Input, 0);
        let output_idx = graph.insert(NodeType::Output, 1);

        // Test attaching and detaching
        graph.attach(input_idx, output_idx);
        assert!(graph[input_idx].outgoing().contains(&output_idx));
        assert!(graph[output_idx].incoming().contains(&input_idx));

        graph.detach(input_idx, output_idx);
        assert!(!graph[input_idx].outgoing().contains(&output_idx));
        assert!(!graph[output_idx].incoming().contains(&input_idx));

        // Test detaching non-existent connection
        graph.detach(input_idx, output_idx); // Should not panic
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_graph_eval_serde() {
        use crate::Eval;

        let mut graph = Graph::default();

        graph.insert(NodeType::Input, 0);
        graph.insert(NodeType::Vertex, 1);
        graph.insert(NodeType::Output, 2);
        graph.attach(0, 1);
        graph.attach(1, 2);

        let serialized = serde_json::to_string(&graph).unwrap();
        let deserialized: Graph<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(graph, deserialized);

        let values = vec![
            (NodeType::Input, vec![Op::var(0), Op::var(1)]),
            (NodeType::Edge, vec![Op::weight()]),
            (NodeType::Vertex, vec![Op::sub(), Op::mul(), Op::linear()]),
            (NodeType::Output, vec![Op::linear()]),
        ];

        let op_graph = Graph::directed(2, 2, values);
        let eval_one = op_graph.eval(&vec![vec![0.5, 1.5]]);

        let serialized_op = serde_json::to_string(&op_graph).unwrap();
        let deserialized_op: Graph<Op<f32>> = serde_json::from_str(&serialized_op).unwrap();

        let deserialized_eval = deserialized_op.eval(&vec![vec![0.5, 1.5]]);

        assert_eq!(eval_one, deserialized_eval);
        assert_eq!(op_graph, deserialized_op);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_graph_pre_built_serde() {
        use crate::Eval;

        let mut graph = Graph::<Op<f32>>::default();

        let idx_one = graph.insert(NodeType::Input, Op::var(0));
        let idx_two = graph.insert(NodeType::Input, Op::constant(5_f32));
        let idx_three = graph.insert(NodeType::Vertex, Op::add());
        let idx_four = graph.insert(NodeType::Output, Op::linear());

        graph
            .attach(idx_one, idx_three)
            .attach(idx_two, idx_three)
            .attach(idx_three, idx_four);

        let eval_to_six_one = graph.eval(&vec![vec![1_f32]]);
        let eval_to_seven_one = graph.eval(&vec![vec![2_f32]]);
        let eval_to_eight_one = graph.eval(&vec![vec![3_f32]]);

        assert_eq!(eval_to_six_one, &[&[6_f32]]);
        assert_eq!(eval_to_seven_one, &[&[7_f32]]);
        assert_eq!(eval_to_eight_one, &[&[8_f32]]);
        assert_eq!(graph.len(), 4);

        let serialized = serde_json::to_string(&graph).unwrap();
        let deserialized: Graph<Op<f32>> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(graph, deserialized);

        let eval_to_six_two = deserialized.eval(&vec![vec![1_f32]]);
        let eval_to_seven_two = deserialized.eval(&vec![vec![2_f32]]);
        let eval_to_eight_two = deserialized.eval(&vec![vec![3_f32]]);

        assert_eq!(eval_to_six_two, &[&[6_f32]]);
        assert_eq!(eval_to_seven_two, &[&[7_f32]]);
        assert_eq!(eval_to_eight_two, &[&[8_f32]]);
        assert_eq!(deserialized.len(), 4);
    }
}
