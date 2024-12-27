use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;

pub mod expr;
pub mod gene_node;
pub mod graph_node;
pub mod tree_node;

pub use expr::*;
pub use gene_node::*;
pub use graph_node::*;
pub use tree_node::*;

use crate::NodeType;

pub trait NodeBehavior {
    type Value;
    type Node: NodeBehavior;
    fn node_type(&self) -> NodeType;
    fn id(&self) -> Uuid;
    fn value(&self) -> &Self::Value;
}

pub type NodeCellSequence<T> = Arc<Vec<NodeCell<T>>>;
pub type NodePermutations<T> = Option<Arc<RefCell<Vec<T>>>>;

#[derive(Debug, Clone, PartialEq)]
pub enum Arity {
    Nullary,
    Unary,
    Binary,
    Ternary,
    Nary,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NodeCell<T> {
    value: T,
    id: Uuid,
    arity: Option<Arity>,
    node_type: NodeType,
    permutations: NodePermutations<T>,
}

impl<T> NodeCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            arity: None,
            id: Uuid::new_v4(),
            node_type: NodeType::Unknown,
            permutations: None,
        }
    }

    pub fn with_node_type(mut self, node_type: NodeType) -> Self {
        self.node_type = node_type;
        self
    }

    pub fn with_permutations(mut self, permutations: NodePermutations<T>) -> Self {
        self.permutations = permutations;
        self
    }

    pub fn with_arity(mut self, arity: Arity) -> Self {
        self.arity = Some(arity);
        self
    }

    pub fn with_value(mut self, value: T) -> Self {
        self.value = value;
        self
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }

    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn new_instance(&self) -> NodeCell<T>
    where
        T: Clone,
    {
        if let Some(permutations) = &self.permutations {
            let perms = permutations.borrow();
            let values = perms.deref();
            let idx = rand::random::<usize>() % values.len();
            let value = values[idx].clone();

            NodeCell {
                value,
                id: Uuid::new_v4(),
                arity: self.arity.clone(),
                node_type: self.node_type,
                permutations: Some(Arc::clone(permutations)),
            }
        } else {
            NodeCell {
                value: self.value.clone(),
                id: Uuid::new_v4(),
                arity: self.arity.clone(),
                node_type: self.node_type,
                permutations: self.permutations.clone(),
            }
        }
    }
}
