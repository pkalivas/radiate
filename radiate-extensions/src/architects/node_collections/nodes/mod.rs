use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;

pub mod expr;
pub mod gene_node;
pub mod graph_node;
pub mod indexed_node;
pub mod op;
pub mod schema;
pub mod tree_node;
pub mod values;

pub use expr::*;
pub use gene_node::*;
pub use graph_node::*;
pub use indexed_node::*;
pub use op::*;
pub use schema::*;
pub use tree_node::*;
pub use values::*;

use crate::NodeType;

pub trait NodeBehavior {
    type Value;
    type Node: NodeBehavior;
    fn node_type(&self) -> NodeType;
    fn id(&self) -> Uuid;
}

pub type NodeCellSequence<T> = Arc<Vec<NodeCell<T>>>;

pub struct NodeCell<T> {
    id: Uuid,
    node_type: NodeType,
    value: T,
}

impl<T> NodeCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            node_type: NodeType::Unknown,
            value,
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> NodeBehavior for NodeCell<T> {
    type Value = T;
    type Node = NodeCell<T>;

    fn node_type(&self) -> NodeType {
        self.node_type
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
        self.value == other.value && self.node_type == other.node_type
    }
}
