use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;
use uuid::Uuid;

pub mod expr;
pub mod gene_node;
pub mod flat_node;
pub mod tree_node;

pub use expr::*;
pub use gene_node::*;
pub use flat_node::*;
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
pub type NodePermutations<T> = Option<Arc<RefCell<Vec<Expr<T>>>>>;

#[derive(Debug, Clone, PartialEq)]
pub enum Arity {
    Nullary,
    Unary,
    Binary,
    Ternary,
    Nary,
}

#[derive(Clone, PartialEq)]
pub struct NodeCell<T> {
    pub value: Expr<T>,
    pub id: Uuid,
    pub arity: Option<Arity>,
    pub node_type: NodeType,
    pub permutations: NodePermutations<T>,
}

impl<T> NodeCell<T> {
    pub fn new(value: Expr<T>) -> Self {
        Self {
            value,
            arity: None,
            id: Uuid::new_v4(),
            node_type: NodeType::Unknown,
            permutations: None,
        }
    }

    pub fn with_arity(mut self, arity: Arity) -> Self {
        self.arity = Some(arity);
        self
    }

    pub fn with_node_type(mut self, node_type: NodeType) -> Self {
        self.node_type = node_type;
        self
    }

    fn new_instance(&self) -> NodeCell<T>
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

    fn cell(&self) -> &NodeCell<T> {
        self
    }

    fn cell_mut(&mut self) -> &mut NodeCell<T> {
        self
    }
}
