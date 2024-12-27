use std::collections::HashSet;
use uuid::Uuid;

use crate::{Direction, NodeCell, NodeType};

use super::NodeBehavior;

pub struct GraphNode<T> {
    cell: NodeCell<T>,
    index: usize,
    enabled: bool,
    direction: Direction,
    incoming: HashSet<usize>,
    outgoing: HashSet<usize>,
}

impl<T> GraphNode<T> {
    pub fn new(index: usize, value: T) -> Self {
        Self {
            cell: NodeCell::new(value),
            index,
            enabled: true,
            direction: Direction::Forward,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
}

impl<T> NodeBehavior for GraphNode<T>
where
    T: Clone + PartialEq + Default,
{
    type Value = T;
    type Node = GraphNode<T>;

    fn node_type(&self) -> NodeType {
        self.cell.node_type()
    }

    fn id(&self) -> Uuid {
        self.cell.id()
    }

    fn value(&self) -> &Self::Value {
        self.cell.value()
    }
}

impl<T> Clone for GraphNode<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
            index: self.index,
            enabled: self.enabled,
            direction: self.direction,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }
}

impl<T> PartialEq for GraphNode<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
            && self.enabled == other.enabled
            && self.direction == other.direction
    }
}
