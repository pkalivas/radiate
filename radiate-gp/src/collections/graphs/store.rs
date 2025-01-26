use crate::collections::{GraphNode, NodeType};
use crate::{Factory, Op};

use radiate::random_provider;
use std::collections::HashMap;

pub struct CellStore<T> {
    values: HashMap<NodeType, Vec<Op<T>>>,
}

impl<T> CellStore<T> {
    pub fn new() -> Self {
        CellStore {
            values: HashMap::new(),
        }
    }

    pub fn add_values(&mut self, node_type: NodeType, values: Vec<Op<T>>) {
        self.values.insert(node_type, values);
    }

    pub fn get_values(&self, node_type: NodeType) -> Option<&Vec<Op<T>>> {
        self.values.get(&node_type)
    }
}

impl<T: Clone> Clone for CellStore<T> {
    fn clone(&self) -> Self {
        let mut store = CellStore::new();
        for (node_type, values) in &self.values {
            store.add_values(*node_type, values.clone());
        }

        store
    }
}

impl<T: PartialEq> PartialEq for CellStore<T> {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl<T: Default + Clone> Factory<GraphNode<T>> for CellStore<T> {
    type Input = (usize, NodeType);

    fn new_instance(&self, input: Self::Input) -> GraphNode<T> {
        let (index, node_type) = input;
        if let Some(values) = self.get_values(node_type) {
            if node_type == NodeType::Input {
                let new_instance = values[index % values.len()].new_instance(());
                return GraphNode::new(index, node_type, new_instance);
            } else {
                let new_instance = random_provider::choose(values).new_instance(());
                return GraphNode::new(index, node_type, new_instance);
            }
        }

        GraphNode::new(index, node_type, Op::default())
    }
}
