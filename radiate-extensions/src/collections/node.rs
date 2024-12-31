use crate::{ops::Arity, Op};

pub trait Node<C: NodeCell> {
    fn arity(&self) -> Arity;
    fn cell(&self, index: usize) -> &C;
    fn cell_mut(&mut self, index: usize) -> &mut C;
}

pub trait NodeCell {
    type Value;

    fn arity(&self) -> Arity;
    fn value(&self) -> &Self::Value;
    fn value_mut(&mut self) -> &mut Self::Value;
    fn new_instance(&self) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// Nodes intended for graphs
    Input,
    Output,
    Vertex,
    Edge,

    /// Nodes intended for trees
    Root,
    Branch,
    Leaf,
}

impl<T: Clone> NodeCell for Op<T> {
    type Value = Self;

    fn arity(&self) -> Arity {
        self.arity()
    }

    fn value(&self) -> &Self::Value {
        self
    }

    fn value_mut(&mut self) -> &mut Self::Value {
        self
    }

    fn new_instance(&self) -> Self::Value {
        self.new_instance()
    }
}