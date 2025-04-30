use super::transaction::TransactionResult;
use crate::collections::graphs::GraphTransaction;
use crate::collections::{Direction, GraphNode};
use crate::{Node, NodeType};
use radiate_core::Valid;
use std::collections::{HashSet, VecDeque};
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

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
/// It also provides methods for iterating over the nodes in the graph in a sudo topological order.
//
#[derive(Clone, PartialEq)]
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

    pub fn inputs(&self) -> Vec<&GraphNode<T>> {
        self.get_nodes_of_type(NodeType::Input)
    }

    pub fn outputs(&self) -> Vec<&GraphNode<T>> {
        self.get_nodes_of_type(NodeType::Output)
    }

    pub fn vertices(&self) -> Vec<&GraphNode<T>> {
        self.get_nodes_of_type(NodeType::Vertex)
    }

    pub fn edges(&self) -> Vec<&GraphNode<T>> {
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
        self.as_mut()[incoming].outgoing_mut().insert(outgoing);
        self.as_mut()[outgoing].incoming_mut().insert(incoming);
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
        self.as_mut()[incoming].outgoing_mut().remove(&outgoing);
        self.as_mut()[outgoing].incoming_mut().remove(&incoming);
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

    /// Get the cycles in the graph that include the node at the specified index.
    ///
    /// # Arguments
    /// - index: The index of the node to get the cycles for.
    #[inline]
    pub fn get_cycles(&self, index: usize) -> Vec<usize> {
        let mut path = Vec::new();
        let mut seen = HashSet::new();
        let mut current = self
            .get(index)
            .map(|node| node.outgoing().iter().cloned().collect::<VecDeque<usize>>())
            .unwrap_or_default();

        while !current.is_empty() {
            let current_index = current.pop_front().unwrap();
            if let Some(current_node) = self.get(current_index) {
                if seen.contains(&current_index) {
                    continue;
                }

                if current_index == index {
                    path.push(current_index);
                    return path;
                }

                seen.insert(current_index);

                if !current_node.outgoing().is_empty() {
                    path.push(current_index);
                    for outgoing in current_node.outgoing().iter() {
                        current.push_back(*outgoing);
                    }
                }
            }
        }

        Vec::new()
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
            let node_cycles = self.get_cycles(idx);

            if node_cycles.is_empty() {
                if let Some(node) = self.get_mut(idx) {
                    node.set_direction(Direction::Forward);
                }
            } else {
                for cycle_idx in node_cycles {
                    if let Some(node) = self.get_mut(cycle_idx) {
                        node.set_direction(Direction::Backward);
                    }
                }
            }
        }
    }

    #[inline]
    fn get_nodes_of_type(&self, node_type: NodeType) -> Vec<&GraphNode<T>> {
        self.nodes
            .iter()
            .filter(|node| node.node_type() == node_type)
            .collect()
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
    use std::collections::BTreeSet;

    #[test]
    fn test_graph_is_valid() {
        let mut graph_one = Graph::default();
        graph_one.push((0, NodeType::Input, 0));
        graph_one.push((1, NodeType::Output, 1));
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

        assert_eq!(graph[0].outgoing(), &BTreeSet::from_iter(vec![1]));
        assert_eq!(graph[1].incoming(), &BTreeSet::from_iter(vec![0]));
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
}
