use crate::architects::node_collections::node::Node;
use crate::{
    architects::schema::node_types::NodeType,
    operations::op::{self, Ops},
};
use radiate::random_provider;
use std::collections::HashMap;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct NodeFactory<T>
where
    T: Clone + PartialEq + Default,
{
    pub node_values: HashMap<NodeType, Vec<Ops<T>>>,
}

impl<T> NodeFactory<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new() -> Self {
        NodeFactory {
            node_values: HashMap::new(),
        }
    }

    pub fn leafs(mut self, values: Vec<Ops<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Leaf, values);
        self
    }

    pub fn inputs(mut self, values: Vec<Ops<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Input, values);
        self
    }

    pub fn outputs(mut self, values: Vec<Ops<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Output, values);
        self
    }

    pub fn gates(mut self, values: Vec<Ops<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Gate, values);
        self
    }

    pub fn aggregates(mut self, values: Vec<Ops<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Aggregate, values);
        self
    }

    pub fn weights(mut self, values: Vec<Ops<T>>) -> NodeFactory<T> {
        self.add_node_values(NodeType::Weight, values);
        self
    }

    pub fn set_values(mut self, node_type: NodeType, values: Vec<Ops<T>>) -> NodeFactory<T> {
        self.add_node_values(node_type, values);
        self
    }

    pub fn add_node_values(&mut self, node_type: NodeType, values: Vec<Ops<T>>) {
        self.node_values.insert(node_type, values);
    }

    pub fn new_node(&self, index: usize, node_type: NodeType) -> Node<T> {
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

        Node::new(index, node_type, Ops::default())
    }

    pub fn regression(input_size: usize) -> NodeFactory<f32> {
        let inputs = (0..input_size).map(op::var).collect::<Vec<Ops<f32>>>();
        NodeFactory::new()
            .inputs(inputs.clone())
            .leafs(inputs.clone())
            .gates(vec![
                op::add(),
                op::sub(),
                op::mul(),
                op::div(),
                op::pow(),
                op::sqrt(),
                op::exp(),
                op::abs(),
                op::log(),
                op::sin(),
                op::cos(),
                op::tan(),
                op::sum(),
                op::prod(),
                op::max(),
                op::min(),
                op::ceil(),
                op::floor(),
                op::gt(),
                op::lt(),
            ])
            .aggregates(vec![
                op::sigmoid(),
                op::tanh(),
                op::relu(),
                op::linear(),
                op::sum(),
                op::prod(),
                op::max(),
                op::min(),
                op::mish(),
                op::leaky_relu(),
                op::softplus(),
                op::sum(),
                op::prod(),
            ])
            .weights(vec![op::weight()])
            .outputs(vec![op::linear()])
    }
}
