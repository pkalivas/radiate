// use crate::collections::NodeType;
use crate::{Arity, Op};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

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
}

impl<T> NodeStore<T> {
    pub fn new() -> Self {
        NodeStore {
            values: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn insert(&self, values: Vec<T>)
    where
        T: Into<NodeValue<T>>,
    {
        let mut values_map = self.values.write().unwrap();

        let valid_values = values
            .into_iter()
            .map(|val| val.into())
            .collect::<Vec<NodeValue<T>>>();

        values_map.extend(valid_values);
    }

    pub fn map<F, K>(&self, arity: Option<Arity>, mapper: F) -> Option<K>
    where
        F: Fn(Vec<&NodeValue<T>>) -> K,
    {
        let reader = self.values.read().unwrap();
        if let Some(arity) = arity {
            let values = reader
                .iter()
                .filter(|value| value.arity() == Some(arity))
                .collect::<Vec<&NodeValue<T>>>();

            if values.is_empty() {
                return None;
            }

            return Some(mapper(values));
        }

        Some(mapper(reader.iter().collect()))
    }
}

impl<T> From<Vec<T>> for NodeStore<T>
where
    T: Into<NodeValue<T>>,
{
    fn from(values: Vec<T>) -> Self {
        let store = NodeStore::new();
        store.insert(values);

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

        store.insert(input_values);
        store.insert(output_values);
        store.insert(edge_values);
        store.insert(node_values);

        store
    }
}

impl<T: Clone> From<&NodeStore<T>> for NodeStore<T> {
    fn from(store: &NodeStore<T>) -> Self {
        NodeStore {
            values: Arc::clone(&store.values),
        }
    }
}

impl<T> Clone for NodeStore<T> {
    fn clone(&self) -> Self {
        NodeStore {
            values: Arc::clone(&self.values),
        }
    }
}

impl<T: PartialEq> PartialEq for NodeStore<T> {
    fn eq(&self, other: &Self) -> bool {
        let self_values = self.values.read().unwrap();
        let other_values = other.values.read().unwrap();

        (*self_values) == (*other_values)
    }
}

impl<T: Debug> Debug for NodeStore<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let values = self.values.read().unwrap();
        write!(f, "{:?}", values)
    }
}
