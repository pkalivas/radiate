use crate::collections::{GraphNode, NodeType};
use crate::ops::Arity;
use crate::{Builder, Factory, Op, Store};
use radiate::random_provider;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

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

    pub fn map_values<F, K>(&self, node_type: NodeType, size: usize, mapper: F) -> Vec<K>
    where
        T: Clone + Default,
        F: Fn(usize, &T) -> K,
    {
        // if node_type == NodeType::Input && !self.contains(NodeType::Input) {
        //     let inputs = (0..size).map(Op::var).collect::<Vec<Op<T>>>();
        //     self.add_values(NodeType::Input, inputs);
        // }

        let reader = self.values.read().unwrap();
        if let Some(values) = reader.get(&node_type) {
            return (0..size)
                .map(|i| {
                    if i < values.len() {
                        return mapper(i, &values[i]);
                    } else {
                        return mapper(i, &values[i % values.len()]);
                    }
                })
                .collect::<Vec<K>>();
        }

        (0..size)
            .map(|i| mapper(i, &T::default()))
            .collect::<Vec<K>>()
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

impl<T> From<HashMap<NodeType, Vec<T>>> for ValueStore<T> {
    fn from(values: HashMap<NodeType, Vec<T>>) -> Self {
        ValueStore {
            values: Arc::new(RwLock::new(values)),
        }
    }
}

impl<T> From<Vec<(NodeType, Vec<T>)>> for ValueStore<T> {
    fn from(values: Vec<(NodeType, Vec<T>)>) -> Self {
        let mut map = HashMap::new();
        for (node_type, ops) in values {
            map.insert(node_type, ops);
        }

        ValueStore {
            values: Arc::new(RwLock::new(map)),
        }
    }
}

impl<T> From<(NodeType, Vec<T>)> for ValueStore<T> {
    fn from(value: (NodeType, Vec<T>)) -> Self {
        let mut map = HashMap::new();
        map.insert(value.0, value.1);

        ValueStore {
            values: Arc::new(RwLock::new(map)),
        }
    }
}

// impl<T: Clone> From<Vec<Op<T>>> for NodeStore<Op<T>> {
//     fn from(values: Vec<Op<T>>) -> Self {
//         let store = NodeStore::new();

//         let input_values = values
//             .iter()
//             .filter(|op| op.arity() == Arity::Zero)
//             .cloned()
//             .collect::<Vec<Op<T>>>();

//         let output_values = values
//             .iter()
//             .filter(|op| op.arity() == Arity::Any)
//             .cloned()
//             .collect::<Vec<Op<T>>>();

//         let edge_values = values
//             .iter()
//             .filter(|op| op.arity() == Arity::Exact(1))
//             .cloned()
//             .collect::<Vec<Op<T>>>();

//         let node_values = values
//             .iter()
//             .filter(|op| op.arity() != Arity::Zero)
//             .cloned()
//             .collect::<Vec<Op<T>>>();

//         store.add_values(NodeType::Input, input_values);
//         store.add_values(NodeType::Output, output_values);
//         store.add_values(NodeType::Edge, edge_values);
//         store.add_values(NodeType::Vertex, node_values);

//         store
//     }
// }

impl<T: Clone> From<Op<T>> for ValueStore<Op<T>> {
    fn from(value: Op<T>) -> Self {
        let store = ValueStore::new();

        let input_values = vec![Op::var(0)];
        let output_values = vec![value.clone()];
        let edge_values = vec![Op::identity()];
        let node_values = vec![value.clone()];

        store.add_values(NodeType::Input, input_values);
        store.add_values(NodeType::Output, output_values);
        store.add_values(NodeType::Edge, edge_values);
        store.add_values(NodeType::Vertex, node_values);

        store
    }
}

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
