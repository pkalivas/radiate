use crate::{CellSchema, Direction, Expr, NodeCell};
use std::collections::HashSet;

#[derive(Clone, PartialEq)]
pub struct IndexedCell<T> {
    pub inner: NodeCell<T>,
    pub index: usize,
    pub incoming: HashSet<usize>,
    pub outgoing: HashSet<usize>,
}

impl<T> IndexedCell<T> {
    pub fn new(inner: NodeCell<T>, index: usize) -> Self {
        IndexedCell {
            inner,
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

impl<T> CellSchema for IndexedCell<T> {
    type ValueType = T;

    fn value(&self) -> &Expr<Self::ValueType> {
        self.inner.value()
    }

    fn id(&self) -> &uuid::Uuid {
        self.inner.id()
    }

    fn enabled(&self) -> bool {
        self.inner.enabled()
    }

    fn direction(&self) -> Direction {
        self.inner.direction()
    }
}

impl<T> From<IndexedCell<T>> for NodeCell<T> {
    fn from(cell: IndexedCell<T>) -> Self {
        cell.inner
    }
}

impl<T> From<NodeCell<T>> for IndexedCell<T> {
    fn from(cell: NodeCell<T>) -> Self {
        IndexedCell {
            inner: cell,
            index: 0,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }
}
