use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub mod graph_node;
pub mod indexed_node;
pub mod rc_node;
pub mod tree_node;

pub use graph_node::*;
pub use indexed_node::*;
pub use rc_node::*;
pub use tree_node::*;

use crate::NodeType;

pub trait NodeBehavior {
    type Value;
    type Node: NodeBehavior;
    fn node_type(&self) -> NodeType;
    fn id(&self) -> Uuid;
}

pub type RefNodeCell<T> = Rc<RefCell<NodeCell<T>>>;

pub type NodeSequce<T> = Arc<Mutex<Vec<RefNodeCell<T>>>>;

pub struct NodeCell<T> {
    id: Uuid,
    node_type: Option<NodeType>,
    value: T,
}

impl<T> NodeCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_type: None,
            value,
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn set_node_type(&mut self, node_type: NodeType) {
        self.node_type = Some(node_type);
    }
}

impl<T> NodeBehavior for NodeCell<T> {
    type Value = T;
    type Node = NodeCell<T>;

    fn node_type(&self) -> NodeType {
        self.node_type.unwrap()
    }

    fn id(&self) -> Uuid {
        self.id
    }
}

impl<T> Clone for NodeCell<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            node_type: self.node_type,
            value: self.value.clone(),
        }
    }
}

impl<T> PartialEq for NodeCell<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.node_type == other.node_type && self.id == other.id
    }
}
