use crate::{ops::Arity, Operation};

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

impl<T: Clone> NodeCell for Operation<T> {
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
