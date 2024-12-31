use crate::collections::{GraphNode, NodeType};
use crate::ops;
use crate::ops::operation::Operation;

use radiate::random_provider;
use std::collections::HashMap;

use super::NodeCell;

pub trait Factory<T> {
    type Input;
    fn new_instance(&self, input: Self::Input) -> T;
}

#[derive(Default, Clone, PartialEq, Debug)]
pub struct NodeFactory<C: Clone> {
    pub node_values: HashMap<NodeType, Vec<C>>,
}

impl<C: Clone> NodeFactory<C> {
    pub fn new() -> Self {
        NodeFactory {
            node_values: HashMap::new(),
        }
    }

    pub fn inputs(mut self, values: Vec<C>) -> NodeFactory<C> {
        self.add_node_values(NodeType::Input, values);
        self
    }

    pub fn outputs(mut self, values: Vec<C>) -> NodeFactory<C> {
        self.add_node_values(NodeType::Output, values);
        self
    }

    pub fn vertices(mut self, values: Vec<C>) -> NodeFactory<C> {
        self.add_node_values(NodeType::Vertex, values);
        self
    }

    pub fn edges(mut self, values: Vec<C>) -> NodeFactory<C> {
        self.add_node_values(NodeType::Edge, values);
        self
    }

    pub fn add_node_values(&mut self, node_type: NodeType, values: Vec<C>) {
        self.node_values.insert(node_type, values);
    }
}

impl NodeFactory<Operation<f32>> {
    pub fn regression(input_size: usize) -> NodeFactory<Operation<f32>> {
        let inputs = (0..input_size)
            .map(Operation::var)
            .collect::<Vec<Operation<f32>>>();
        NodeFactory::new()
            .inputs(inputs.clone())
            .vertices(ops::get_all_operations())
            .edges(vec![Operation::weight(), Operation::identity()])
            .outputs(vec![Operation::linear()])
    }
}

impl<C: NodeCell + Clone + Default> Factory<GraphNode<C>> for HashMap<NodeType, Vec<C>> {
    type Input = (usize, NodeType);

    fn new_instance(&self, input: Self::Input) -> GraphNode<C> {
        let (index, node_type) = input;
        if let Some(values) = self.get(&node_type) {
            return match node_type {
                NodeType::Input => {
                    // let value = values[index % values.len()].clone();
                    let value = random_provider::choose(values).new_instance();
                    GraphNode::new(index, node_type, value)
                }
                _ => {
                    let value = random_provider::choose(values);
                    GraphNode::new(index, node_type, value.new_instance())
                }
            };
        }

        GraphNode::new(index, node_type, C::default())
    }
}

impl<C> Factory<GraphNode<C>> for NodeFactory<C>
where
    C: Clone + Default + NodeCell,
{
    type Input = (usize, NodeType);

    fn new_instance(&self, input: Self::Input) -> GraphNode<C> {
        let (index, node_type) = input;
        if let Some(values) = self.node_values.get(&node_type) {
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

        GraphNode::new(index, node_type, C::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let factory: NodeFactory<Operation<f32>> = NodeFactory::new();
        assert!(factory.node_values.is_empty());
    }
}
