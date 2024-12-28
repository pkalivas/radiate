use crate::architects::cells::expr::{self, Expr};
use crate::architects::node_collections::node::Node;
use crate::architects::schema::node_types::NodeType;
use crate::{IndexedCell, ValueCell};
use radiate::random_provider;
use std::collections::HashMap;
use std::rc::Rc;

type ValueStore<T> = HashMap<NodeType, Rc<Vec<Expr<T>>>>;

#[derive(Clone, Default, PartialEq, Debug)]
pub struct ValueFactory<T>
where
    T: Clone + PartialEq + Default,
{
    values: ValueStore<T>,
}

impl<T> ValueFactory<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new() -> Self {
        ValueFactory {
            values: HashMap::new(),
        }
    }

    pub fn leafs(mut self, values: Vec<Expr<T>>) -> ValueFactory<T> {
        self.add_node_values(NodeType::Leaf, values);
        self
    }

    pub fn inputs(mut self, values: Vec<Expr<T>>) -> ValueFactory<T> {
        self.add_node_values(NodeType::Input, values);
        self
    }

    pub fn outputs(mut self, values: Vec<Expr<T>>) -> ValueFactory<T> {
        self.add_node_values(NodeType::Output, values);
        self
    }

    pub fn gates(mut self, values: Vec<Expr<T>>) -> ValueFactory<T> {
        self.add_node_values(NodeType::Gate, values);
        self
    }

    pub fn aggregates(mut self, values: Vec<Expr<T>>) -> ValueFactory<T> {
        self.add_node_values(NodeType::Aggregate, values);
        self
    }

    pub fn weights(mut self, values: Vec<Expr<T>>) -> ValueFactory<T> {
        self.add_node_values(NodeType::Weight, values);
        self
    }

    pub fn set_values(mut self, node_type: NodeType, values: Vec<Expr<T>>) -> ValueFactory<T> {
        self.add_node_values(node_type, values);
        self
    }

    pub fn add_node_values(&mut self, node_type: NodeType, values: Vec<Expr<T>>) {
        self.values.insert(node_type, Rc::new(values));
    }

    pub fn new_node(&self, index: usize, node_type: NodeType) -> Node<T> {
        if let Some(values) = self.values.get(&node_type) {
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

    pub fn new_value(&self, node_type: NodeType) -> ValueCell<T> {
        if let Some(values) = self.values.get(&node_type) {
            return random_provider::choose(values).new_instance().into();
        }

        ValueCell::default()
    }

    pub fn new_indexed(&self, index: usize, node_type: NodeType) -> IndexedCell<T> {
        if let Some(values) = self.values.get(&node_type) {
            match node_type {
                NodeType::Input => {
                    let value = values[index % values.len()].clone();
                    return IndexedCell::new(value.into(), index);
                }
                _ => {
                    let value = random_provider::choose(values);
                    return IndexedCell::new(value.new_instance().into(), index);
                }
            }
        }

        IndexedCell::default()
    }

    pub fn regression(input_size: usize) -> ValueFactory<f32> {
        let inputs = (0..input_size).map(expr::var).collect::<Vec<Expr<f32>>>();
        ValueFactory::new()
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
        let factory: ValueFactory<Expr<f32>> = ValueFactory::new();
        assert!(factory.values.is_empty());
    }
}
