use crate::Arity;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Input,
    Output,
    Vertex,
    Edge,
    Leaf,
    Root,
}

pub trait Node {
    type Value;

    fn value(&self) -> &Self::Value;
    fn node_type(&self) -> NodeType;
    fn arity(&self) -> Arity;
}
