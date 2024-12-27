use std::{cell::RefCell, rc::Rc};

use uuid::Uuid;

use crate::NodeType;

use super::{NodeBehavior, NodeCell, RefNodeCell};

/// A wrapper around a node that provides a reference counted pointer to the node.
/// This is useful for creating a collection of nodes that can be shared across multiple
/// structures.
pub struct RcNode<T> {
    inner: RefNodeCell<T>,
}

impl<T> RcNode<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(NodeCell::new(value))),
        }
    }

    pub fn inner(&self) -> RefNodeCell<T> {
        Rc::clone(&self.inner)
    }
}

impl<N> Clone for RcNode<N>
where
    N: NodeBehavior,
{
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<N> NodeBehavior for RcNode<N>
where
    N: NodeBehavior,
{
    type Value = N::Value;
    type Node = N;

    fn node_type(&self) -> NodeType {
        self.inner.borrow().node_type()
    }

    fn id(&self) -> Uuid {
        self.inner.borrow().id()
    }
}
