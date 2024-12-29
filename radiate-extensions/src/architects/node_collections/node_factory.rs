use crate::architects::node_collections::node::GraphNode;
use crate::ops::operation;
use crate::ops::operation::Operation;
use crate::NodeType;
use radiate::random_provider;
use std::collections::HashMap;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct NodeFactory<T: Clone> {
    pub node_values: HashMap<NodeType, Vec<Operation<T>>>,
}

impl<T> NodeFactory<T>
where
    T: Clone,
{
    pub fn new() -> Self {
        NodeFactory {
            node_values: HashMap::new(),
        }
    }

    pub fn inputs(mut self, values: Vec<Operation<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Input, values);
        self
    }

    pub fn outputs(mut self, values: Vec<Operation<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Output, values);
        self
    }

    pub fn vertices(mut self, values: Vec<Operation<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Vertex, values);
        self
    }

    pub fn edges(mut self, values: Vec<Operation<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Edge, values);
        self
    }

    pub fn add_node_values(&mut self, node_type: NodeType, values: Vec<Operation<T>>) {
        self.node_values.insert(node_type, values);
    }

    pub fn new_node(&self, index: usize, node_type: NodeType) -> GraphNode<T>
    where
        T: Default,
    {
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

        GraphNode::new(index, node_type, Operation::default())
    }

    pub fn regression(input_size: usize) -> NodeFactory<f32> {
        let inputs = (0..input_size)
            .map(operation::var)
            .collect::<Vec<Operation<f32>>>();
        NodeFactory::new()
            .inputs(inputs.clone())
            .vertices(vec![
                // operation::add(),
                // operation::sub(),
                // operation::mul(),
                // operation::div(),
                // operation::pow(),
                // operation::sqrt(),
                // operation::exp(),
                // operation::abs(),
                // operation::log(),
                // operation::sin(),
                // operation::cos(),
                // operation::tan(),
                // operation::ceil(),
                // operation::floor(),
                // operation::gt(),
                // operation::lt(),
                operation::sigmoid(),
                operation::tanh(),
                operation::relu(),
                operation::linear(),
                operation::max(),
                operation::min(),
                operation::mish(),
                operation::leaky_relu(),
                operation::softplus(),
                operation::sum(),
                operation::prod(),
            ])
            .edges(vec![operation::weight(), operation::identity()])
            .outputs(vec![operation::linear()])
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
