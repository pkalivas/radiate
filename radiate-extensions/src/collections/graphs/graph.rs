use std::collections::{HashSet, VecDeque};
use std::fmt::Debug;
use std::ops::{Index, IndexMut};

use super::{GraphIterator, NodeType};
use crate::collections::graphs::GraphTransaction;
use crate::collections::{Direction, GraphNode};
use crate::NodeCell;

use radiate::{random_provider, Valid};

/// A 'Graph' is simply a 'Vec' of 'GraphNode's.
///
/// Its important to note that this graph differs from a traditional graph in that it is not
/// a collection of edges and vertices. Instead, it is a collection of nodes that are connected
/// to one another. Each node has a unique index that is used to reference it in the graph
/// and must be identical to its position in the 'Vec'.
/// Each 'GraphNode' has a 'HashSet' of incoming and outgoing connections. These connections are
/// represented by the index of the connected node in the graph. Because of this representation,
/// an edge is not a separate entity, its just a node. The 'NodeType' enum is used to distinguish
/// different types of nodes. This allows for a more flexible representation of the graph
/// while still maintaining the ability to represent traditional graphs.
///
/// By default, a 'Graph' is a directed acyclic graph (DAG). However, it is possible to create
/// cycles in the graph by setting the 'direction' field of a 'GraphNode' to 'Direction::Backward'.
/// The 'Graph' struct provides methods for attaching and detaching nodes from one another.
/// It also provides methods for iterating over the nodes in the graph in a sudo topological order.
//
#[derive(Clone, Default)]
pub struct Graph<C: NodeCell> {
    nodes: Vec<GraphNode<C>>,
}

/// The 'Graph' struct provides methods for creating, modifying, and iterating over a graph.
impl<C: NodeCell> Graph<C> {
    /// Create a new 'Graph' from a 'Vec' of 'GraphNode's.
    ///
    /// # Arguments
    /// - nodes: A 'Vec' of 'GraphNode's.
    pub fn new(nodes: Vec<GraphNode<C>>) -> Self {
        Graph { nodes }
    }

    /// Push a 'GraphNode' onto the last position in the graph.
    pub fn push(&mut self, node: GraphNode<C>) {
        self.nodes.push(node);
    }

    pub fn add<T>(&mut self, node_type: NodeType, val: T) -> usize
    where
        T: Into<C>,
    {
        let node = GraphNode::new(self.len(), node_type, val.into());
        self.push(node);
        self.len() - 1
    }

    /// Pop the last 'GraphNode' from the graph.
    pub fn pop(&mut self) -> Option<GraphNode<C>> {
        self.nodes.pop()
    }

    /// Returns the number of nodes in the graph.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns true if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns a mutable reference to the node at the specified index.
    pub fn get_mut(&mut self, index: usize) -> &mut GraphNode<C> {
        self.nodes.get_mut(index).unwrap()
    }

    /// Returns a reference to the node at the specified index.
    pub fn get(&self, index: usize) -> &GraphNode<C> {
        self.nodes.get(index).unwrap()
    }

    /// iterates over the nodes in the graph. The nodes are returned in the order they
    /// were added, so there is no real order to this iterator.
    pub fn iter(&self) -> impl Iterator<Item = &GraphNode<C>> {
        self.nodes.iter()
    }
    /// mutably iterates over the nodes in the graph. The nodes are returned in the order they
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut GraphNode<C>> {
        self.nodes.iter_mut()
    }

    /// iterates over the nodes in the graph in a sudo topological order. This means that
    /// the nodes are returned in an order that respects the connections between them.
    /// It is a 'best effort' topological order, as it is not guaranteed to be a true topological
    /// order. This is because the graph may contain cycles which would make it impossible to
    /// create a true topological order.
    pub fn topological_iter(&self) -> impl Iterator<Item = &GraphNode<C>> {
        GraphIterator::new(self)
    }

    /// Attach and detach nodes from one another. This is the primary way to modify the graph.
    /// Note that this method does not check if the nodes are already connected. This is because
    /// the connections are represented by 'HashSet's which do not allow duplicates.
    /// Its also important to note that the 'incoming' and 'outgoing' indices are the indices of the
    /// nodes in the graph, not the indices of the connections in the 'incoming' and 'outgoing' 'HashSet's.
    /// We must also remember that the 'GraphNode' cares about the 'Arity' of the 'Operation' it contains,
    /// so if we add a connection that would violate the 'Arity' of the 'Operation', the connection will result
    /// in a 'GraphNode' that is not 'Valid'.
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
}

/// Functinos for modifying the graph.
impl<C: NodeCell> Graph<C> {
    /// Given a list of node indices, this function will set the 'direction' field of the nodes
    /// at those indices to 'Direction::Backward' if they are part of a cycle. If they are not part
    /// of a cycle, the 'direction' field will be set to 'Direction::Forward'.
    /// If no indices are provided, the function will set the 'direction' field of all nodes in the graph.
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
                let node = self.get_mut(idx);
                node.set_direction(Direction::Forward);
            } else {
                for cycle_idx in node_cycles {
                    let node = self.get_mut(cycle_idx);
                    node.set_direction(Direction::Backward);
                }
            }
        }
    }

    /// tries to modify the graph using a 'GraphTransaction'. If the transaction is successful,
    /// we return true and do nothing. If the transaction is not successful, we rollback the transaction
    /// by undoing all the changes made by the transaction and return false.
    ///
    /// # Arguments
    ///  - mutation: A closure that takes a mutable reference to a 'GraphTransaction' and returns a 'bool'.
    pub fn try_modify<F>(&mut self, mutation: F) -> bool
    where
        F: FnOnce(&mut GraphTransaction<C>) -> bool,
        C: Clone + Default + PartialEq,
    {
        let mut transaction = GraphTransaction::new(self);
        if !mutation(&mut transaction) {
            transaction.rollback();
            return false;
        }

        true
    }
}

/// Functions for checking the validity of the graph or connections between nodes. These are
/// useful for modifying the graph in a way that maintains its integrity.
impl<C: NodeCell> Graph<C> {
    /// Get the cycles in the graph that include the node at the specified index.
    ///
    /// # Arguments
    /// - index: The index of the node to get the cycles for.
    #[inline]
    pub fn get_cycles(&self, index: usize) -> Vec<usize> {
        let mut path = Vec::new();
        let mut seen = HashSet::new();
        let mut current = self[index]
            .incoming()
            .iter()
            .cloned()
            .collect::<VecDeque<usize>>();

        while !current.is_empty() {
            let current_index = current.pop_front().unwrap();
            let current_node = &self[current_index];

            if seen.contains(&current_index) {
                continue;
            }

            if current_index == index {
                return path;
            }

            seen.insert(current_index);

            if !current_node.incoming().is_empty() {
                path.push(current_index);
                for outgoing in current_node.incoming().iter() {
                    current.push_back(*outgoing);
                }
            }
        }

        Vec::new()
    }
    /// Check if two nodes can be connected. This is determined by a few rules:
    /// - The source node must have outgoing connections or be recurrent.
    /// - The source and target nodes must not be edges.
    /// - The source and target nodes must not be the same.
    /// - The connection must not create a cycle if it is not recurrent.
    ///
    /// If all these conditions are met, the function will return true. Otherwise, it will return false.
    ///
    /// # Arguments
    /// - source: The index of the source node.
    /// - target: The index of the target node.
    /// - recurrent: A flag that indicates if the desired connection is recurrent.
    #[inline]
    pub fn can_connect(&self, source: usize, target: usize, recurrent: bool) -> bool {
        let source_node = &self[source];
        let target_node = &self[target];

        if (source_node.outgoing().is_empty() || source_node.is_recurrent()) && !recurrent {
            return false;
        }

        let would_create_cycle = recurrent || !self.would_create_cycle(source, target);
        let nodes_are_weights =
            source_node.node_type() == NodeType::Edge || target_node.node_type() == NodeType::Edge;

        would_create_cycle && !nodes_are_weights && source != target
    }
    /// Check if connecting the source node to the target node would create a cycle.
    ///
    /// # Arguments
    /// - source: The index of the source node.
    /// - target: The index of the target node.
    ///
    #[inline]
    pub fn would_create_cycle(&self, source: usize, target: usize) -> bool {
        let mut seen = HashSet::new();
        let mut visited = self.get(target).outgoing().iter().collect::<Vec<&usize>>();

        while !visited.is_empty() {
            let node_index = visited.pop().unwrap();

            seen.insert(*node_index);

            if *node_index == source {
                return true;
            }

            for edge_index in self
                .get(*node_index)
                .outgoing()
                .iter()
                .filter(|edge_index| !seen.contains(edge_index))
            {
                visited.push(edge_index);
            }
        }

        false
    }
    /// The below functinos are used to get random nodes from the graph. These are useful for
    /// creating connections between nodes. Neither of these functions will return an edge node.
    /// This is because edge nodes are not valid source or target nodes for connections as they
    /// they only allow one incoming and one outgoing connection, thus they can't be used to create
    /// new connections. Instread, edge nodes are used to represent the weights of the connections
    ///
    /// Get a random node that can be used as a source node for a connection.
    /// A source node can be either an input or a vertex node.
    #[inline]
    pub fn random_source_node(&self) -> &GraphNode<C> {
        self.random_node_of_type(vec![NodeType::Input, NodeType::Vertex])
    }
    /// Get a random node that can be used as a target node for a connection.
    /// A target node can be either an output or a vertex node.
    #[inline]
    pub fn random_target_node(&self) -> &GraphNode<C> {
        self.random_node_of_type(vec![NodeType::Output, NodeType::Vertex])
    }
    /// Helper functions to get a random node of the specified type. If no nodes of the specified
    /// type are found, the function will try to get a random node of a different type.
    /// If no nodes are found, the function will panic.
    #[inline]
    fn random_node_of_type(&self, node_types: Vec<NodeType>) -> &GraphNode<C> {
        if node_types.is_empty() {
            panic!("At least one node type must be specified.");
        }

        let gene_node_type_index = random_provider::random::<usize>() % node_types.len();
        let gene_node_type = node_types.get(gene_node_type_index).unwrap();

        let genes = match gene_node_type {
            NodeType::Input => self
                .iter()
                .filter(|node| node.node_type() == NodeType::Input)
                .collect::<Vec<&GraphNode<C>>>(),
            NodeType::Output => self
                .iter()
                .filter(|node| node.node_type() == NodeType::Output)
                .collect::<Vec<&GraphNode<C>>>(),
            NodeType::Vertex => self
                .iter()
                .filter(|node| node.node_type() == NodeType::Vertex)
                .collect::<Vec<&GraphNode<C>>>(),
            NodeType::Edge => self
                .iter()
                .filter(|node| node.node_type() == NodeType::Edge)
                .collect::<Vec<&GraphNode<C>>>(),
        };

        if genes.is_empty() {
            return self.random_node_of_type(
                node_types
                    .iter()
                    .filter(|nt| *nt != gene_node_type)
                    .cloned()
                    .collect(),
            );
        }

        let index = random_provider::random::<usize>() % genes.len();
        genes.get(index).unwrap()
    }
}

impl<C> Valid for Graph<C>
where
    C: NodeCell + Clone + PartialEq + Default,
{
    fn is_valid(&self) -> bool {
        self.iter().all(|node| node.is_valid())
    }
}

impl<C: NodeCell> AsRef<[GraphNode<C>]> for Graph<C> {
    fn as_ref(&self) -> &[GraphNode<C>] {
        &self.nodes
    }
}

impl<C: NodeCell> AsMut<[GraphNode<C>]> for Graph<C> {
    fn as_mut(&mut self) -> &mut [GraphNode<C>] {
        &mut self.nodes
    }
}

impl<C: NodeCell> Index<usize> for Graph<C> {
    type Output = GraphNode<C>;

    fn index(&self, index: usize) -> &Self::Output {
        self.nodes.get(index).expect("Index out of bounds.")
    }
}

impl<C: NodeCell> IndexMut<usize> for Graph<C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.nodes.get_mut(index).expect("Index out of bounds.")
    }
}

impl<C: NodeCell> IntoIterator for Graph<C> {
    type Item = GraphNode<C>;
    type IntoIter = std::vec::IntoIter<GraphNode<C>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<C: NodeCell + PartialEq> PartialEq for Graph<C> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

impl<C: NodeCell + Debug + PartialEq + Clone> Debug for Graph<C> {
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

    #[test]
    fn test_simple_graph() {
        let mut graph = Graph::<i32>::default();

        graph.add(NodeType::Input, 0);
        graph.add(NodeType::Vertex, 1);
        graph.add(NodeType::Output, 2);

        graph.attach(0, 1).attach(1, 2);

        println!("{:?}", graph);

        assert_eq!(graph.len(), 3);

        assert!(graph.is_valid());
        assert!(graph.get(0).is_valid());
        assert!(graph.get(1).is_valid());
        assert!(graph.get(2).is_valid());

        assert_eq!(graph.get(0).incoming().len(), 0);
        assert_eq!(graph.get(0).outgoing().len(), 1);
        assert_eq!(graph.get(1).incoming().len(), 1);
        assert_eq!(graph.get(1).outgoing().len(), 1);
        assert_eq!(graph.get(2).incoming().len(), 1);
        assert_eq!(graph.get(2).outgoing().len(), 0);
    }

    #[test]
    fn test_graph_with_cycles() {
        let mut graph = Graph::<i32>::default();

        graph.add(NodeType::Input, 0);
        graph.add(NodeType::Vertex, 1);
        graph.add(NodeType::Vertex, 2);
        graph.add(NodeType::Output, 3);

        graph.attach(0, 1).attach(1, 2).attach(2, 1).attach(2, 3);

        println!("{:?}", graph);

        assert_eq!(graph.len(), 4);

        assert!(graph.is_valid());
        assert!(graph.get(0).is_valid());
        assert!(graph.get(1).is_valid());
        assert!(graph.get(2).is_valid());
        assert!(graph.get(3).is_valid());

        assert_eq!(graph.get(0).incoming().len(), 0);
        assert_eq!(graph.get(0).outgoing().len(), 1);
        assert_eq!(graph.get(1).incoming().len(), 2);
        assert_eq!(graph.get(1).outgoing().len(), 1);
        assert_eq!(graph.get(2).incoming().len(), 1);
        assert_eq!(graph.get(2).outgoing().len(), 2);
        assert_eq!(graph.get(3).incoming().len(), 1);
        assert_eq!(graph.get(3).outgoing().len(), 0);
    }

    #[test]
    fn test_graph_with_cycles_and_recurrent_nodes() {
        let mut graph = Graph::<i32>::default();

        graph.add(NodeType::Input, 0);
        graph.add(NodeType::Vertex, 1);
        graph.add(NodeType::Vertex, 2);
        graph.add(NodeType::Output, 3);

        graph
            .attach(0, 1)
            .attach(1, 2)
            .attach(2, 1)
            .attach(2, 3)
            .attach(3, 1);

        println!("{:?}", graph);

        graph.set_cycles(vec![]);

        println!("{:?}", graph);

        assert_eq!(graph.len(), 4);

        assert!(graph.is_valid());
        assert!(graph.get(0).is_valid());
        assert!(graph.get(1).is_valid());
        assert!(graph.get(2).is_valid());
        assert!(graph.get(3).is_valid());

        assert_eq!(graph.get(0).incoming().len(), 0);
        assert_eq!(graph.get(0).outgoing().len(), 1);
        assert_eq!(graph.get(1).incoming().len(), 3);
        assert_eq!(graph.get(1).outgoing().len(), 1);
        assert_eq!(graph.get(2).incoming().len(), 1);
        assert_eq!(graph.get(2).outgoing().len(), 2);
        assert_eq!(graph.get(3).incoming().len(), 1);
        assert_eq!(graph.get(3).outgoing().len(), 1);

        assert_eq!(graph.get(0).direction(), Direction::Forward);
        assert_eq!(graph.get(1).direction(), Direction::Backward);
        assert_eq!(graph.get(2).direction(), Direction::Backward);
        assert_eq!(graph.get(3).direction(), Direction::Backward);
    }
}
