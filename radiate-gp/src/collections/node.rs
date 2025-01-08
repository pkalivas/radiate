use crate::ops::Arity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Input,
    Output,
    Vertex,
    Edge,
}

pub trait NodeCell {
    fn arity(&self) -> Arity;
    fn new_instance(&self) -> Self;
}

impl NodeCell for i32 {
    fn arity(&self) -> Arity {
        Arity::Any
    }

    fn new_instance(&self) -> Self {
        *self
    }
}
