use std::collections::HashSet;

use crate::Direction;

use super::NodeCell;

#[derive(Clone, PartialEq)]
pub struct GraphNode<T> {
    pub cell: NodeCell<T>,
    pub enabled: bool,
    pub direction: Direction,
    pub index: usize,
    pub incoming: HashSet<usize>,
    pub outgoing: HashSet<usize>,
}

impl<T> GraphNode<T> {
    pub fn new(index: usize, cell: NodeCell<T>) -> Self {
        GraphNode {
            cell,
            index,
            enabled: true,
            direction: Direction::Forward,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
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
}

impl<T> AsRef<NodeCell<T>> for GraphNode<T> {
    fn as_ref(&self) -> &NodeCell<T> {
        &self.cell
    }
}

impl<T> AsMut<NodeCell<T>> for GraphNode<T> {
    fn as_mut(&mut self) -> &mut NodeCell<T> {
        &mut self.cell
    }
}
