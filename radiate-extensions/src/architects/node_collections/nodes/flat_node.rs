use crate::{Direction, Expr, NodeCell};
use std::collections::HashSet;

pub struct FlatNode<T> {
    pub cell: NodeCell<T>,
    pub index: usize,
    pub enabled: bool,
    pub direction: Direction,
    pub incoming: HashSet<usize>,
    pub outgoing: HashSet<usize>,
}

impl<T> FlatNode<T> {
    pub fn new(index: usize, value: Expr<T>) -> Self {
        Self {
            cell: NodeCell::new(value),
            index,
            enabled: true,
            direction: Direction::Forward,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }

    pub fn from_cell(cell: NodeCell<T>, index: usize) -> Self {
        Self {
            cell,
            index,
            enabled: true,
            direction: Direction::Forward,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }
}

impl<T> Clone for FlatNode<T>
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

impl<T> PartialEq for FlatNode<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
            && self.enabled == other.enabled
            && self.direction == other.direction
    }
}
