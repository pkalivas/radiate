use crate::architects::cells::expr::{self, Expr};
use crate::architects::node_collections::node::Node;
use crate::architects::schema::node_types::NodeType;
use radiate::random_provider;
use std::collections::HashMap;

#[derive(Default, Clone, PartialEq, Debug)]
pub struct NodeFactory<T: Clone> {
    pub node_values: HashMap<NodeType, Vec<Expr<T>>>,
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

    pub fn leafs(mut self, values: Vec<Expr<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Leaf, values);
        self
    }

    pub fn inputs(mut self, values: Vec<Expr<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Input, values);
        self
    }

    pub fn outputs(mut self, values: Vec<Expr<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Output, values);
        self
    }

    pub fn gates(mut self, values: Vec<Expr<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Gate, values);
        self
    }

    pub fn aggregates(mut self, values: Vec<Expr<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Aggregate, values);
        self
    }

    pub fn weights(mut self, values: Vec<Expr<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Weight, values);
        self
    }

    pub fn set_values(mut self, node_type: NodeType, values: Vec<Expr<T>>) -> NodeFactory<T> {
        self.add_node_values(node_type, values);
        self
    }

    pub fn add_node_values(&mut self, node_type: NodeType, values: Vec<Expr<T>>) {
        self.node_values.insert(node_type, values);
    }

    pub fn new_node(&self, index: usize, node_type: NodeType) -> Node<T>
    where
        T: Default,
    {
        if let Some(values) = self.node_values.get(&node_type) {
            return match node_type {
                NodeType::Input => {
                    let value = values[index % values.len()].clone();
                    Node::new(index, node_type, value)
                }
                _ => {
                    let value = random_provider::choose(values);
                    Node::new(index, node_type, value.new_instance())
                }
            };
        }

        Node::new(index, node_type, Expr::default())
    }

    pub fn regression(input_size: usize) -> NodeFactory<f32> {
        let inputs = (0..input_size).map(expr::var).collect::<Vec<Expr<f32>>>();
        NodeFactory::new()
            .inputs(inputs.clone())
            .leafs(inputs.clone())
            .gates(vec![
                expr::add(),
                expr::sub(),
                expr::mul(),
                expr::div(),
                expr::pow(),
                expr::sqrt(),
                expr::exp(),
                expr::abs(),
                expr::log(),
                expr::sin(),
                expr::cos(),
                expr::tan(),
                expr::sum(),
                expr::prod(),
                expr::max(),
                expr::min(),
                expr::ceil(),
                expr::floor(),
                expr::gt(),
                expr::lt(),
            ])
            .aggregates(vec![
                expr::sigmoid(),
                expr::tanh(),
                expr::relu(),
                expr::linear(),
                expr::sum(),
                expr::prod(),
                expr::max(),
                expr::min(),
                expr::mish(),
                expr::leaky_relu(),
                expr::softplus(),
                expr::sum(),
                expr::prod(),
            ])
            .weights(vec![expr::weight()])
            .outputs(vec![expr::linear()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let factory: NodeFactory<Expr<f32>> = NodeFactory::new();
        assert!(factory.node_values.is_empty());
    }
}
