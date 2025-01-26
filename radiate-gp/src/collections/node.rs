use crate::{node_cell, ops::Arity};

pub trait NodeCell {
    // fn arity(&self) -> Arity;
    // fn new_instance(&self) -> Self;
}

node_cell!(f32, Arity::Any);
node_cell!(i32, Arity::Any);
node_cell!(bool, Arity::Any);
node_cell!(String, Arity::Any);
node_cell!(char, Arity::Any);
