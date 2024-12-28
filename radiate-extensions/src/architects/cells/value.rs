use crate::Expr;
use std::collections::HashSet;

#[derive(Clone, PartialEq)]
pub struct Value<T> {
    pub value: Expr<T>,
    pub id: uuid::Uuid,
}

impl<T> Value<T> {
    pub fn new(value: Expr<T>) -> Self {
        Value {
            value,
            id: uuid::Uuid::new_v4(),
        }
    }
}

impl<T> AsRef<Expr<T>> for Value<T> {
    fn as_ref(&self) -> &Expr<T> {
        &self.value
    }
}

impl<T> AsRef<Value<T>> for Value<T> {
    fn as_ref(&self) -> &Value<T> {
        self
    }
}

impl<T> From<Value<T>> for Expr<T> {
    fn from(cell: Value<T>) -> Self {
        cell.value
    }
}

impl<T> From<Expr<T>> for Value<T> {
    fn from(cell: Expr<T>) -> Self {
        Value {
            value: cell,
            id: uuid::Uuid::new_v4(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct IndexedValue<T> {
    pub inner: Value<T>,
    pub index: usize,
    pub incoming: HashSet<usize>,
    pub outgoing: HashSet<usize>,
}

impl<T> IndexedValue<T> {
    pub fn new(inner: Value<T>, index: usize) -> Self {
        IndexedValue {
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

impl<T> AsRef<Value<T>> for IndexedValue<T> {
    fn as_ref(&self) -> &Value<T> {
        &self.inner
    }
}

impl<T> From<IndexedValue<T>> for Value<T> {
    fn from(cell: IndexedValue<T>) -> Self {
        cell.inner
    }
}

impl<T> From<Value<T>> for IndexedValue<T> {
    fn from(cell: Value<T>) -> Self {
        IndexedValue {
            inner: cell,
            index: 0,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }
}
