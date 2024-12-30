pub mod architect;
pub mod builder;

use std::collections::HashMap;

pub use architect::*;
pub use builder::*;

use radiate::random_provider;

use crate::ops::Operation;

use super::{Graph, GraphNode, NodeType};

pub trait Builder {
    type Output;
    fn build(&self) -> Self::Output;
}

pub trait Factory<T> {
    type Input;
    fn new_instance(&self, input: Self::Input) -> T;
}

impl<T> Factory<GraphNode<T>> for HashMap<NodeType, Vec<Operation<T>>>
where
    T: Clone + Default,
{
    type Input = (usize, NodeType);

    fn new_instance(&self, input: Self::Input) -> GraphNode<T> {
        let (index, node_type) = input;
        if let Some(values) = self.get(&node_type) {
            return match node_type {
                NodeType::Input => {
                    let value = values[index % values.len()].clone();
                    GraphNode::new(index, node_type, value)
                }
                _ => {
                    let value = random_provider::choose(values);
                    GraphNode::new(index, node_type, value.new_instance())
                }
            };
        }

        GraphNode::new(index, node_type, Operation::default())
    }
}
