use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, RwLock},
};

use radiate::random_provider;

use crate::{Factory, GraphNode, NodeType, Store};

use super::{Arity, Op};

pub struct OpStore<K: Eq + Hash, T> {
    store: Arc<RwLock<HashMap<K, Vec<Op<T>>>>>,
}

impl<K: Eq + Hash, T> OpStore<K, T> {
    pub fn new() -> Self {
        OpStore {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn insert(&self, key: K, ops: Vec<Op<T>>) {
        let mut writer = self.store.write().unwrap();
        writer.insert(key, ops);
    }

    pub fn get(&self, key: &K) -> Option<Vec<Op<T>>>
    where
        T: Clone,
    {
        let reader = self.store.read().unwrap();
        reader.get(key).cloned()
    }

    pub fn contains(&self, key: &K) -> bool {
        let reader = self.store.read().unwrap();
        reader.contains_key(key) && !reader[key].is_empty()
    }
}

impl<K: Eq + Hash, T> Store<K, Vec<Op<T>>> for OpStore<K, T> {
    fn map<F, R>(&self, key: K, f: F) -> Option<R>
    where
        F: Fn(&Vec<Op<T>>) -> R,
    {
        let reader = self.store.read().unwrap();
        reader.get(&key).map(f)
    }

    fn get_or_insert_with<F>(&self, key: K, f: F) -> Vec<Op<T>>
    where
        Vec<Op<T>>: Clone,
        F: Fn() -> Vec<Op<T>>,
    {
        let mut writer = self.store.write().unwrap();
        writer.entry(key).or_insert_with(f).clone()
    }
}

impl<T: Clone + Default> Factory<GraphNode<Op<T>>> for OpStore<NodeType, T> {
    type Input = (usize, NodeType);

    fn new_instance(&self, input: Self::Input) -> GraphNode<Op<T>> {
        let (index, key) = input;

        let new_node = self.map(key, |values| {
            let new_value = match key {
                NodeType::Input => values[index % values.len()].clone(),
                _ => random_provider::choose(values).new_instance(()),
            };

            let mut node = GraphNode::<Op<T>>::new(index, key, new_value);
            node.set_arity(node.value().arity());

            node
        });

        if let Some(new_value) = new_node {
            return new_value;
        }

        GraphNode::new(index, key, Op::default())
    }
}

impl<K: Eq + Hash, T> Clone for OpStore<K, T> {
    fn clone(&self) -> Self {
        OpStore {
            store: Arc::clone(&self.store),
        }
    }
}

impl<K: Eq + Hash, T: PartialEq> PartialEq for OpStore<K, T> {
    fn eq(&self, other: &Self) -> bool {
        let self_store = self.store.read().unwrap();
        let other_store = other.store.read().unwrap();

        (*self_store) == (*other_store)
    }
}

impl<K: Eq + Hash, T> From<HashMap<K, Vec<Op<T>>>> for OpStore<K, T> {
    fn from(store: HashMap<K, Vec<Op<T>>>) -> Self {
        OpStore {
            store: Arc::new(RwLock::new(store)),
        }
    }
}

impl<K: Eq + Hash, T> From<Vec<(K, Vec<Op<T>>)>> for OpStore<K, T> {
    fn from(store: Vec<(K, Vec<Op<T>>)>) -> Self {
        let mut map = HashMap::new();
        for (key, ops) in store {
            map.insert(key, ops);
        }

        OpStore {
            store: Arc::new(RwLock::new(map)),
        }
    }
}

impl<T: Clone> From<Vec<Op<T>>> for OpStore<NodeType, T> {
    fn from(values: Vec<Op<T>>) -> Self {
        let store = OpStore::new();

        let input_values = values
            .iter()
            .filter(|op| op.arity() == Arity::Zero)
            .cloned()
            .collect::<Vec<Op<T>>>();

        let output_values = values
            .iter()
            .filter(|op| op.arity() == Arity::Any)
            .cloned()
            .collect::<Vec<Op<T>>>();

        let edge_values = values
            .iter()
            .filter(|op| op.arity() == Arity::Exact(1))
            .cloned()
            .collect::<Vec<Op<T>>>();

        let node_values = values
            .iter()
            .filter(|op| op.arity() != Arity::Zero)
            .cloned()
            .collect::<Vec<Op<T>>>();

        store.insert(NodeType::Input, input_values);
        store.insert(NodeType::Output, output_values);
        store.insert(NodeType::Edge, edge_values);
        store.insert(NodeType::Vertex, node_values);

        store
    }
}
