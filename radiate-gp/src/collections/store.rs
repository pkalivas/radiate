use crate::collections::{GraphNode, NodeType};
use crate::ops::operation::Op;
use crate::ops::{self};

use super::{Factory, NodeCell};

use radiate::random_provider;
use std::collections::HashMap;

pub struct CellStore<C: NodeCell> {
    values: HashMap<NodeType, Vec<C>>,
}

impl<C: NodeCell> CellStore<C> {
    pub fn new() -> Self {
        CellStore {
            values: HashMap::new(),
        }
    }

    pub fn add_values(&mut self, node_type: NodeType, values: Vec<C>) {
        self.values.insert(node_type, values);
    }

    pub fn get_values(&self, node_type: NodeType) -> Option<&Vec<C>> {
        self.values.get(&node_type)
    }
}

impl CellStore<Op<f32>> {
    pub fn regression(input_size: usize) -> CellStore<Op<f32>> {
        let inputs = (0..input_size).map(Op::var).collect::<Vec<Op<f32>>>();
        let mut store = CellStore::new();

        store.add_values(NodeType::Input, inputs);
        store.add_values(NodeType::Vertex, ops::get_all_operations());
        store.add_values(NodeType::Edge, vec![Op::weight(), Op::identity()]);
        store.add_values(NodeType::Output, vec![Op::linear()]);

        store
    }

    pub fn classification(input_size: usize) -> CellStore<Op<f32>> {
        let inputs = (0..input_size).map(Op::var).collect::<Vec<Op<f32>>>();
        let mut store = CellStore::new();

        store.add_values(NodeType::Input, inputs);
        store.add_values(NodeType::Vertex, ops::get_all_operations());
        store.add_values(NodeType::Edge, vec![Op::weight(), Op::identity()]);
        store.add_values(NodeType::Output, vec![Op::sigmoid()]);

        store
    }
}

impl<C: NodeCell> Default for CellStore<C> {
    fn default() -> Self {
        CellStore::new()
    }
}

impl<C: NodeCell + Clone> Clone for CellStore<C> {
    fn clone(&self) -> Self {
        let mut store = CellStore::new();
        for (node_type, values) in &self.values {
            store.add_values(*node_type, values.clone());
        }

        store
    }
}

impl<C: NodeCell + PartialEq> PartialEq for CellStore<C> {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl<C: NodeCell + Default> Factory<GraphNode<C>> for CellStore<C> {
    type Input = (usize, NodeType);

    fn new_instance(&self, input: Self::Input) -> GraphNode<C> {
        let (index, node_type) = input;
        if let Some(values) = self.get_values(node_type) {
            let new_instance = random_provider::choose(values).new_instance();
            return GraphNode::new(index, node_type, new_instance);
        }

        GraphNode::new(index, node_type, C::default())
    }
}
