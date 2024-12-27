use std::{collections::HashSet, sync::Arc};

use uuid::Uuid;

use crate::NodeType;

use super::{NodeBehavior, NodeSequce, RefNodeCell};

/// A node that is indexed in a collection of nodes.
/// This is useful for creating a graph of nodes where each node can be referenced by its index.
/// The indexed node contains the index of the node in the collection, the collection of nodes,
/// and the incoming and outgoing nodes.
/// The incoming and outgoing nodes are stored as a set of indexes.
///
/// # Type Parameters
/// - `N`: The type of the node behavior.
///
pub struct IndexedNode<T> {
    index: usize,
    nodes: NodeSequce<T>,
    incoming: HashSet<usize>,
    outgoing: HashSet<usize>,
}

impl<T> IndexedNode<T> {
    pub fn new(index: usize, nodes: NodeSequce<T>) -> Self {
        Self {
            index,
            nodes,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn incoming(&self) -> &HashSet<usize> {
        &self.incoming
    }

    pub fn outgoing(&self) -> &HashSet<usize> {
        &self.outgoing
    }

    pub fn incoming_mut(&mut self) -> &mut HashSet<usize> {
        &mut self.incoming
    }

    pub fn outgoing_mut(&mut self) -> &mut HashSet<usize> {
        &mut self.outgoing
    }

    pub fn node(&self) -> RefNodeCell<T> {
        self.nodes.lock().unwrap()[self.index].clone()
    }
}

impl<N> NodeBehavior for IndexedNode<N>
where
    N: NodeBehavior,
{
    type Value = N::Value;
    type Node = N;

    fn node_type(&self) -> NodeType {
        self.node().borrow().node_type()
    }

    fn id(&self) -> Uuid {
        self.node().borrow().id()
    }
}

impl<T> Clone for IndexedNode<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index,
            nodes: Arc::clone(&self.nodes),
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }
}

impl<T> PartialEq for IndexedNode<T> {
    fn eq(&self, other: &Self) -> bool {
        let self_node = self.node();
        let other_node = other.node();

        self.index == other.index
            && self_node.borrow().node_type() == other_node.borrow().node_type()
            && self_node.borrow().id() == other_node.borrow().id()
    }
}
