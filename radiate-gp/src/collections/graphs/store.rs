use crate::collections::{GraphNode, NodeType};
use crate::{Factory, Store};
use radiate::random_provider;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use super::NodeBuilder;

pub trait NodeStore<T>: Factory<GraphNode<T>, Input = (usize, NodeType)> {}

impl<T: Default + Clone> NodeStore<T> for ValueStore<T> {}

// impl<T: Default + Clone + 'static> Into<NodeBuilder<T>> for ValueStore<T> {
//     fn into(self) -> NodeBuilder<T> {
//         NodeBuilder::new(Arc::new(self))
//     }
// }

pub struct ValueStore<T> {
    values: Arc<RwLock<HashMap<NodeType, Vec<T>>>>,
}

impl<T> ValueStore<T> {
    pub fn new() -> Self {
        ValueStore {
            values: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_values(&self, node_type: NodeType, values: Vec<T>) {
        let mut values_map = self.values.write().unwrap();
        values_map.insert(node_type, values);
    }

    pub fn contains(&self, node_type: NodeType) -> bool {
        let values = self.values.read().unwrap();
        values.contains_key(&node_type) && !values[&node_type].is_empty()
    }

    pub fn map<F, K>(&self, node_type: NodeType, mapper: F) -> Option<K>
    where
        F: Fn(&Vec<T>) -> K,
    {
        let reader = self.values.read().unwrap();
        if let Some(values) = reader.get(&node_type) {
            if values.is_empty() {
                return None;
            }

            return Some(mapper(values));
        }

        None
    }
}

impl<T> Store<NodeType, Vec<T>> for ValueStore<T> {
    fn map<F, R>(&self, key: NodeType, f: F) -> Option<R>
    where
        F: Fn(&Vec<T>) -> R,
    {
        self.map(key, f)
    }

    fn get_or_insert_with<F>(&self, key: NodeType, f: F) -> Vec<T>
    where
        Vec<T>: Clone,
        F: Fn() -> Vec<T>,
    {
        let mut writer = self.values.write().unwrap();
        writer.entry(key).or_insert_with(f).clone()
    }
}

impl<T: Default + Clone> Factory<GraphNode<T>> for ValueStore<T> {
    type Input = (usize, NodeType);

    fn new_instance(&self, input: Self::Input) -> GraphNode<T> {
        let (index, node_type) = input;

        let new_node = self.map(node_type, |values| {
            let new_value = match node_type {
                NodeType::Input => values[index % values.len()].clone(),
                _ => random_provider::choose(values).clone(),
            };

            GraphNode::new(index, node_type, new_value)
        });

        if let Some(new_value) = new_node {
            return new_value;
        }

        GraphNode::new(index, node_type, T::default())
    }
}

impl<T> Clone for ValueStore<T> {
    fn clone(&self) -> Self {
        ValueStore {
            values: Arc::clone(&self.values),
        }
    }
}

impl<T: PartialEq> PartialEq for ValueStore<T> {
    fn eq(&self, other: &Self) -> bool {
        let self_values = self.values.read().unwrap();
        let other_values = other.values.read().unwrap();

        (*self_values) == (*other_values)
    }
}

impl<T: Default + Clone + 'static> Into<ValueStore<T>> for Vec<(NodeType, Vec<T>)> {
    fn into(self) -> ValueStore<T> {
        let map = ValueStore::new();
        for value in self {
            map.add_values(value.0, value.1);
        }

        map
    }
}

impl<T: Default + Clone + 'static> Into<NodeBuilder<T>> for ValueStore<T> {
    fn into(self) -> NodeBuilder<T> {
        NodeBuilder::new(Arc::new(self))
    }
}

// impl<T> From<HashMap<NodeType, Vec<T>>> for ValueStore<T> {
//     fn from(values: HashMap<NodeType, Vec<T>>) -> Self {
//         ValueStore {
//             values: Arc::new(RwLock::new(values)),
//         }
//     }
// }

// impl<T: Clone + Default + 'static> From<(NodeType, Vec<T>)> for NodeBuilder<T> {
//     fn from(value: (NodeType, Vec<T>)) -> Self {
//         let map = ValueStore::new();
//         map.add_values(value.0, value.1);

//         NodeBuilder::new(Arc::new(map))
//     }
// }

// impl<T: Clone + Default + 'static> From<Vec<(NodeType, Vec<T>)>> for NodeBuilder<T> {
//     fn from(values: Vec<(NodeType, Vec<T>)>) -> Self {
//         let map = ValueStore::new();
//         for value in values {
//             map.add_values(value.0, value.1);
//         }

//         NodeBuilder::new(Arc::new(map))
//     }
// }

impl<T: Clone> From<&ValueStore<T>> for ValueStore<T> {
    fn from(store: &ValueStore<T>) -> Self {
        ValueStore {
            values: Arc::clone(&store.values),
        }
    }
}

impl<T: Debug> Debug for ValueStore<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values = self.values.read().unwrap();
        write!(f, "{:?}", values)
    }
}
