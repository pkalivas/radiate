use std::cell::RefCell;
use std::collections::HashSet;
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

pub trait NodeSchema<T> {
    fn with_node_type(&mut self, node_type: NodeType) -> &mut Self {
        self.cell_mut().node_type = node_type;
        self
    }

    fn with_permutations(&mut self, permutations: NodePermutations<T>) -> &mut Self {
        self.cell_mut().permutations = permutations;
        self
    }

    fn with_arity(&mut self, arity: Arity) -> &mut Self {
        self.cell_mut().arity = Some(arity);
        self
    }

    fn with_value(&mut self, value: T) -> &mut Self {
        self.cell_mut().value = value;
        self
    }

    fn value(&self) -> &T {
        &self.cell().value
    }

    fn value_mut(&mut self) -> &mut T {
        &mut self.cell_mut().value
    }

    fn node_type(&self) -> NodeType {
        self.cell().node_type
    }

    fn id(&self) -> Uuid {
        self.cell().id
    }

    fn new_instance(&self) -> Self
    where
        T: Clone;

    fn cell(&self) -> &NodeCell<T>;
    fn cell_mut(&mut self) -> &mut NodeCell<T>;
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
}

impl<T> NodeSchema<T> for NodeCell<T> {
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

pub struct FlatNodeCell<T> {
    cell: NodeCell<T>,
    index: usize,
    incoming: HashSet<usize>,
    outgoing: HashSet<usize>,
}

impl<T> FlatNodeCell<T> {
    pub fn new(index: usize, value: T) -> Self {
        Self {
            cell: NodeCell::new(value),
            index,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }
}

impl<T> NodeSchema<T> for FlatNodeCell<T> {
    fn new_instance(&self) -> FlatNodeCell<T>
    where
        T: Clone,
    {
        FlatNodeCell {
            cell: self.cell.new_instance(),
            index: self.index,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }

    fn cell(&self) -> &NodeCell<T> {
        &self.cell
    }
    fn cell_mut(&mut self) -> &mut NodeCell<T> {
        &mut self.cell
    }
}

impl<T> Clone for FlatNodeCell<T>
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

impl<T> PartialEq for FlatNodeCell<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.cell == other.cell
    }
}
