use uuid::Uuid;

use crate::{Direction, IndexedNode, NodeType};

use super::NodeBehavior;

pub struct GraphNode<T>
where
    T: Clone + PartialEq + Default,
{
    cell: IndexedNode<T>,
    enabled: bool,
    direction: Direction,
}

impl<T> GraphNode<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(cell: IndexedNode<T>) -> Self {
        Self {
            cell,
            enabled: true,
            direction: Direction::Forward,
        }
    }

    pub fn cell(&self) -> &IndexedNode<T> {
        &self.cell
    }

    pub fn cell_mut(&mut self) -> &mut IndexedNode<T> {
        &mut self.cell
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
        self.cell.node().borrow().node_type()
    }

    fn id(&self) -> Uuid {
        self.cell.node().borrow().id()
    }
}

impl<T> Clone for GraphNode<T>
where
    T: Clone + PartialEq + Default,
{
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
            enabled: self.enabled,
            direction: self.direction,
        }
    }
}

impl<T> PartialEq for GraphNode<T>
where
    T: Clone + PartialEq + Default,
{
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
            && self.enabled == other.enabled
            && self.direction == other.direction
    }
}
