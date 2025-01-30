// use crate::collections::NodeType;
use crate::{Arity, Op};
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

use super::node::Node;
use super::NodeType;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeValue<T> {
    Bounded(T, Arity),
    Unbound(T),
}

impl<T> NodeValue<T> {
    pub fn value(&self) -> &T {
        match self {
            NodeValue::Bounded(value, _) => value,
            NodeValue::Unbound(value) => value,
        }
    }

    pub fn arity(&self) -> Option<Arity> {
        match self {
            NodeValue::Bounded(_, arity) => Some(*arity),
            NodeValue::Unbound(_) => None,
        }
    }
}

pub struct NodeStore<T> {
    values: Arc<RwLock<Vec<NodeValue<T>>>>,
    node_type_lookup: Arc<RwLock<HashMap<NodeType, Vec<usize>>>>,
    arity_lookup: Arc<RwLock<HashMap<Arity, Vec<usize>>>>,
}

impl<T> NodeStore<T> {
    pub fn new() -> Self {
        NodeStore {
            values: Arc::new(RwLock::new(Vec::new())),
            node_type_lookup: Arc::new(RwLock::new(HashMap::new())),
            arity_lookup: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, node_type: NodeType, values: Vec<T>)
    where
        T: Into<NodeValue<T>>,
    {
        let mut index_lookup = self.node_type_lookup.write().unwrap();
        let mut store_values = self.values.write().unwrap();
        let mut arity_lookup = self.arity_lookup.write().unwrap();

        for value in values {
            let node_value = value.into();

            index_lookup
                .entry(node_type)
                .or_insert_with(Vec::new)
                .push(store_values.len());
            arity_lookup
                .entry(node_value.arity().unwrap_or(Arity::Any))
                .or_insert_with(Vec::new)
                .push(store_values.len());
            store_values.push(node_value);
        }
    }

    pub fn map_by_arity<F, K>(&self, arity: Arity, mapper: F) -> Option<K>
    where
        F: Fn(Vec<&NodeValue<T>>) -> K,
    {
        let indecies = self.arity_lookup.read().unwrap();
        let values = self.values.read().unwrap();

        if let Some(indices) = indecies.get(&arity) {
            let mapped = indices
                .iter()
                .map(|index| values.get(*index).unwrap())
                .collect::<Vec<&NodeValue<T>>>();

            return Some(mapper(mapped));
        }

        None
    }

    pub fn map<F, K>(&self, mapper: F) -> Option<K>
    where
        F: Fn(&Vec<NodeValue<T>>) -> K,
    {
        let values = self.values.read().unwrap();
        Some(mapper(&values))
    }

    pub fn filter_map<F, M, K>(&self, filter: F, mapper: M) -> Option<K>
    where
        F: Fn(&NodeValue<T>) -> bool,
        M: Fn(Vec<&NodeValue<T>>) -> K,
    {
        let reader = self.values.read().unwrap();
        let filtered = reader
            .iter()
            .filter(|value| filter(value))
            .collect::<Vec<&NodeValue<T>>>();

        if !filtered.is_empty() {
            return Some(mapper(filtered));
        }

        None
    }

    pub fn map_by<F, K>(&self, node_type: NodeType, mapper: F) -> Option<K>
    where
        F: Fn(Vec<&NodeValue<T>>) -> K,
    {
        let indecies = self.node_type_lookup.read().unwrap();
        let values = self.values.read().unwrap();
        if let Some(indices) = indecies.get(&node_type) {
            let mapped = indices
                .iter()
                .map(|index| values.get(*index).unwrap())
                .collect::<Vec<&NodeValue<T>>>();

            return Some(mapper(mapped));
        }

        None
    }

    pub fn map_by_type<F, K>(&self, node_type: NodeType, mapper: F) -> Option<K>
    where
        F: Fn(Vec<&NodeValue<T>>) -> K,
    {
        let indecies = self.node_type_lookup.read().unwrap();
        let values = self.values.read().unwrap();

        if let Some(indices) = indecies.get(&node_type) {
            let mapped = indices
                .iter()
                .map(|index| values.get(*index).unwrap())
                .collect::<Vec<&NodeValue<T>>>();

            return Some(mapper(mapped));
        }

        None
    }
}

impl<T> From<HashMap<NodeType, Vec<T>>> for NodeStore<T>
where
    T: Into<NodeValue<T>>,
{
    fn from(values: HashMap<NodeType, Vec<T>>) -> Self {
        let store = NodeStore::new();
        for (node_type, ops) in values {
            store.insert(node_type, ops);
        }

        store
    }
}

impl<T> From<Vec<(NodeType, Vec<T>)>> for NodeStore<T>
where
    T: Into<NodeValue<T>>,
{
    fn from(values: Vec<(NodeType, Vec<T>)>) -> Self {
        let store = NodeStore::new();
        for (node_type, ops) in values {
            store.insert(node_type, ops);
        }

        store
    }
}

impl<T> From<Vec<T>> for NodeStore<T>
where
    T: Into<NodeValue<T>> + Clone,
{
    fn from(values: Vec<T>) -> Self {
        let store = NodeStore::new();

        store.insert(NodeType::Input, values.clone());
        store.insert(NodeType::Vertex, values.clone());
        store.insert(NodeType::Output, values.clone());
        store.insert(NodeType::Edge, values.clone());

        store
    }
}

impl<T: Clone> From<Op<T>> for NodeStore<Op<T>> {
    fn from(value: Op<T>) -> Self {
        let store = NodeStore::new();

        let input_values = vec![Op::var(0)];
        let output_values = vec![value.clone()];
        let edge_values = vec![Op::identity()];
        let node_values = vec![value.clone()];

        store.insert(NodeType::Input, input_values);
        store.insert(NodeType::Output, output_values);
        store.insert(NodeType::Edge, edge_values);
        store.insert(NodeType::Vertex, node_values);

        store
    }
}

impl<T: Clone> From<&NodeStore<T>> for NodeStore<T> {
    fn from(store: &NodeStore<T>) -> Self {
        NodeStore {
            node_type_lookup: Arc::clone(&store.node_type_lookup),
            values: Arc::clone(&store.values),
            arity_lookup: Arc::clone(&store.arity_lookup),
        }
    }
}

impl<T> Clone for NodeStore<T> {
    fn clone(&self) -> Self {
        NodeStore {
            node_type_lookup: Arc::clone(&self.node_type_lookup),
            values: Arc::clone(&self.values),
            arity_lookup: Arc::clone(&self.arity_lookup),
        }
    }
}

impl<T: PartialEq> PartialEq for NodeStore<T> {
    fn eq(&self, other: &Self) -> bool {
        let self_values = self.node_type_lookup.read().unwrap();
        let other_values = other.node_type_lookup.read().unwrap();

        (*self_values) == (*other_values)
    }
}

impl<T: Debug> Debug for NodeStore<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values = self.node_type_lookup.read().unwrap();
        write!(f, "{:?}", values)
    }
}

// // use crate::collections::NodeType;
// use crate::{Arity, Op};
// use std::collections::HashMap;
// use std::fmt::Debug;
// use std::sync::{Arc, RwLock};

// use super::NodeType;

// #[derive(Debug, Clone, PartialEq)]
// pub enum NodeValue<T> {
//     Bounded(T, Arity),
//     Unbound(T),
// }

// impl<T> NodeValue<T> {
//     pub fn value(&self) -> &T {
//         match self {
//             NodeValue::Bounded(value, _) => value,
//             NodeValue::Unbound(value) => value,
//         }
//     }

//     pub fn arity(&self) -> Option<Arity> {
//         match self {
//             NodeValue::Bounded(_, arity) => Some(*arity),
//             NodeValue::Unbound(_) => None,
//         }
//     }
// }

// pub struct NodeStore<T> {
//     values: Arc<RwLock<HashMap<NodeType, Vec<NodeValue<T>>>>>,
// }

// impl<T> NodeStore<T> {
//     pub fn new() -> Self {
//         NodeStore {
//             values: Arc::new(RwLock::new(HashMap::new())),
//         }
//     }

//     pub fn insert(&self, node_type: NodeType, values: Vec<T>)
//     where
//         T: Into<NodeValue<T>>,
//     {
//         let mut values_map = self.values.write().unwrap();

//         let valid_values = values
//             .into_iter()
//             .map(|val| val.into())
//             .filter(|val| match val {
//                 NodeValue::Unbound(_) => true,
//                 NodeValue::Bounded(_, arity) => {
//                     return match node_type {
//                         NodeType::Input => arity == &Arity::Zero,
//                         NodeType::Output => arity == &Arity::Any,
//                         NodeType::Edge => arity == &Arity::Exact(1),
//                         NodeType::Vertex => arity != &Arity::Zero,
//                         NodeType::Leaf => arity == &Arity::Zero,
//                         NodeType::Root => arity != &Arity::Zero,
//                     }
//                 }
//             })
//             .collect();

//         values_map.insert(node_type, valid_values);
//     }

//     pub fn map_by_type<F, K>(&self, node_type: NodeType, mapper: F) -> Option<K>
//     where
//         F: Fn(&Vec<NodeValue<T>>) -> K,
//     {
//         let reader = self.values.read().unwrap();
//         if let Some(values) = reader.get(&node_type) {
//             return Some(mapper(values));
//         }

//         None
//     }
// }

// impl<T> From<HashMap<NodeType, Vec<T>>> for NodeStore<T>
// where
//     T: Into<NodeValue<T>>,
// {
//     fn from(values: HashMap<NodeType, Vec<T>>) -> Self {
//         let store = NodeStore::new();
//         for (node_type, ops) in values {
//             store.insert(node_type, ops);
//         }

//         store
//     }
// }

// impl<T> From<Vec<(NodeType, Vec<T>)>> for NodeStore<T>
// where
//     T: Into<NodeValue<T>>,
// {
//     fn from(values: Vec<(NodeType, Vec<T>)>) -> Self {
//         let store = NodeStore::new();
//         for (node_type, ops) in values {
//             store.insert(node_type, ops);
//         }

//         store
//     }
// }

// impl<T> From<Vec<T>> for NodeStore<T>
// where
//     T: Into<NodeValue<T>> + Clone,
// {
//     fn from(values: Vec<T>) -> Self {
//         let store = NodeStore::new();

//         store.insert(NodeType::Input, values.clone());
//         store.insert(NodeType::Vertex, values.clone());
//         store.insert(NodeType::Output, values.clone());
//         store.insert(NodeType::Edge, values.clone());

//         store
//     }
// }

// impl<T: Clone> From<Op<T>> for NodeStore<Op<T>> {
//     fn from(value: Op<T>) -> Self {
//         let store = NodeStore::new();

//         let input_values = vec![Op::var(0)];
//         let output_values = vec![value.clone()];
//         let edge_values = vec![Op::identity()];
//         let node_values = vec![value.clone()];

//         store.insert(NodeType::Input, input_values);
//         store.insert(NodeType::Output, output_values);
//         store.insert(NodeType::Edge, edge_values);
//         store.insert(NodeType::Vertex, node_values);

//         store
//     }
// }

// impl<T: Clone> From<&NodeStore<T>> for NodeStore<T> {
//     fn from(store: &NodeStore<T>) -> Self {
//         NodeStore {
//             values: Arc::clone(&store.values),
//         }
//     }
// }

// impl<T> Clone for NodeStore<T> {
//     fn clone(&self) -> Self {
//         NodeStore {
//             values: Arc::clone(&self.values),
//         }
//     }
// }

// impl<T: PartialEq> PartialEq for NodeStore<T> {
//     fn eq(&self, other: &Self) -> bool {
//         let self_values = self.values.read().unwrap();
//         let other_values = other.values.read().unwrap();

//         (*self_values) == (*other_values)
//     }
// }

// impl<T: Debug> Debug for NodeStore<T> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let values = self.values.read().unwrap();
//         write!(f, "{:?}", values)
//     }
// }
