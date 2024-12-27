use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;

pub mod gene_node;
pub mod graph_node;
pub mod indexed_node;
pub mod rc_node;
pub mod schema;
pub mod tree_node;

pub use gene_node::*;
pub use graph_node::*;
pub use indexed_node::*;
pub use rc_node::*;
pub use schema::*;
pub use tree_node::*;

use crate::NodeType;

pub trait NodeBehavior {
    type Value;
    type Node: NodeBehavior;
    fn node_type(&self) -> NodeType;
    fn id(&self) -> Uuid;
}

pub type NodeCellSequence<T> = Arc<Vec<NodeCell<T>>>;

pub struct NodeCell<T> {
    schema: Option<NodeSchema<T>>,
    value: T,
}

impl<T> NodeCell<T> {
    pub fn new(value: T, schema: Option<NodeSchema<T>>) -> Self {
        Self { schema, value }
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
        match &self.schema {
            Some(schema) => schema.node_type.unwrap(),
            None => NodeType::Unknown,
        }
    }

    fn id(&self) -> Uuid {
        match &self.schema {
            Some(schema) => schema.id,
            None => Uuid::new_v4(),
        }
    }
}

impl<T> Clone for NodeCell<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            schema: self.schema.clone(),
            value: self.value.clone(),
        }
    }
}

impl<T> PartialEq for NodeCell<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.schema == other.schema
    }
}
