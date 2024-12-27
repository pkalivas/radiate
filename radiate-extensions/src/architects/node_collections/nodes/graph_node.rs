use crate::{Direction, FlatNodeCell};

pub struct GraphNode<T> {
    cell: FlatNodeCell<T>,
    enabled: bool,
    direction: Direction,
}

impl<T> GraphNode<T> {
    pub fn new(index: usize, value: T) -> Self {
        Self {
            cell: FlatNodeCell::new(index, value),
            enabled: true,
            direction: Direction::Forward,
        }
    }

    pub fn cell(&self) -> &FlatNodeCell<T> {
        &self.cell
    }

    pub fn cell_mut(&mut self) -> &mut FlatNodeCell<T> {
        &mut self.cell
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
}

// impl<T> NodeBehavior for GraphNode<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     type Value = T;
//     type Node = GraphNode<T>;
//
//     fn node_type(&self) -> NodeType {
//         self.cell.node_type()
//     }
//
//     fn id(&self) -> Uuid {
//         self.cell.id()
//     }
//
//     fn value(&self) -> &Self::Value {
//         self.cell.value()
//     }
// }

impl<T> Clone for GraphNode<T>
where
    T: Clone,
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
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
            && self.enabled == other.enabled
            && self.direction == other.direction
    }
}
