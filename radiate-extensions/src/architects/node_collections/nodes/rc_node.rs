use std::rc::Rc;
use std::sync::RwLock;
use uuid::Uuid;

use crate::NodeType;

use super::{NodeBehavior, NodeCell};

/// A wrapper around a node that provides a reference counted pointer to the node.
/// This is useful for creating a collection of nodes that can be shared across multiple
/// structures.
pub struct RcNode<T> {
    inner: Rc<RwLock<NodeCell<T>>>,
}

impl<T> RcNode<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RwLock::new(NodeCell::new(value, None))),
        }
    }

    pub fn inner(&self) -> Rc<RwLock<NodeCell<T>>> {
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

impl<T> NodeBehavior for RcNode<T> {
    type Value = T;
    type Node = RcNode<T>;

    fn node_type(&self) -> NodeType {
        let read = (*self.inner).read().unwrap();
        read.node_type()
    }

    fn id(&self) -> Uuid {
        let read = (*self.inner).read().unwrap();
        read.id()
    }
}