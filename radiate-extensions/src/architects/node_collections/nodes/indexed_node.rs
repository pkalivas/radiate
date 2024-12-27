use crate::{Expr, NodeCell};
use std::collections::HashSet;

pub struct IndexedNode<T> {
    pub cell: NodeCell<T>,
    pub index: usize,
    pub incoming: HashSet<usize>,
    pub outgoing: HashSet<usize>,
}

impl<T> IndexedNode<T> {
    pub fn new(index: usize, value: Expr<T>) -> Self {
        Self {
            cell: NodeCell::new(value),
            index,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }

    pub fn from_cell(cell: NodeCell<T>, index: usize) -> Self {
        Self {
            cell,
            index,
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

impl<T> Clone for IndexedNode<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            cell: self.cell.clone(),
            index: self.index,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }
}

impl<T> PartialEq for IndexedNode<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
    }
}
