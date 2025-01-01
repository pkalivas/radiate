use crate::{ops::Arity, Op};

pub trait NodeCell {
    type Value;

    fn arity(&self) -> Arity;
    fn value(&self) -> &Self::Value;
    fn new_instance(&self) -> Self;
}

impl<T: Clone> NodeCell for Op<T> {
    type Value = Self;

    fn arity(&self) -> Arity {
        self.arity()
    }

    fn value(&self) -> &Self::Value {
        self
    }

    fn new_instance(&self) -> Self::Value {
        self.new_instance()
    }
}

impl NodeCell for i32 {
    type Value = i32;

    fn arity(&self) -> Arity {
        Arity::Any
    }

    fn value(&self) -> &Self::Value {
        self
    }

    fn new_instance(&self) -> Self::Value {
        *self
    }
}
