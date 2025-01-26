use crate::collections::{GraphNode, NodeType};
use crate::{Factory, NodeCell};

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
