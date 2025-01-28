use super::{Arity, Op};
use crate::{
    graphs::{NodeBuilder, NodeStore},
    Factory, GraphNode, NodeType, Store,
};
use radiate::random_provider;
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{Arc, RwLock},
};

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

impl<T: Default + Clone + 'static> NodeStore<Op<T>> for OpStore<NodeType, T> {}

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
        let (index, node_type) = input;

        let new_value: Op<T> = self.new_instance((index, node_type));

        let arity = new_value.arity();
        let mut node = GraphNode::new(index, node_type, new_value);

        node.set_arity(arity);

        node
    }
}

impl<T: Clone + Default> Factory<Op<T>> for OpStore<NodeType, T> {
    type Input = (usize, NodeType);

    fn new_instance(&self, input: Self::Input) -> Op<T> {
        let (index, node_type) = input;

        let new_node = self.map(node_type, |values| match node_type {
            NodeType::Input => values[index % values.len()].clone(),
            _ => random_provider::choose(values).new_instance(()),
        });

        if let Some(new_value) = new_node {
            return new_value;
        }

        Op::default()
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

impl<T: Clone + Default + 'static> From<Op<T>> for OpStore<NodeType, T> {
    fn from(value: Op<T>) -> Self {
        let store = OpStore::new();

        let input_values = vec![Op::var(0)];
        let output_values = vec![value.clone()];
        let edge_values = vec![Op::identity()];
        let node_values = vec![value.clone()];

        store.insert(NodeType::Input, input_values);
        store.insert(NodeType::Output, output_values);
        store.insert(NodeType::Edge, edge_values);
        store.insert(NodeType::Vertex, node_values);

        store.into()
    }
}

impl<T: Clone + Default + 'static> Into<NodeBuilder<Op<T>>> for Vec<Op<T>> {
    fn into(self) -> NodeBuilder<Op<T>> {
        let store = OpStore::new();

        let input_values = self
            .iter()
            .filter(|op| op.arity() == Arity::Zero)
            .cloned()
            .collect::<Vec<Op<T>>>();

        let output_values = self
            .iter()
            .filter(|op| op.arity() == Arity::Any)
            .cloned()
            .collect::<Vec<Op<T>>>();

        let edge_values = self
            .iter()
            .filter(|op| op.arity() == Arity::Exact(1))
            .cloned()
            .collect::<Vec<Op<T>>>();

        let node_values = self
            .iter()
            .filter(|op| op.arity() != Arity::Zero)
            .cloned()
            .collect::<Vec<Op<T>>>();

        store.insert(NodeType::Input, input_values);
        store.insert(NodeType::Output, output_values);
        store.insert(NodeType::Edge, edge_values);
        store.insert(NodeType::Vertex, node_values);

        store.into()
    }
}

impl<T: Clone + Default + 'static> Into<OpStore<NodeType, T>> for Vec<(NodeType, Vec<Op<T>>)> {
    fn into(self) -> OpStore<NodeType, T> {
        let store = OpStore::new();

        for (node_type, ops) in self {
            store.insert(node_type, ops);
        }

        store.into()
    }
    // fn from(values: Vec<(NodeType, Vec<Op<T>>)>) -> Self {
    //     let map = OpStore::new();
    //     for (node_type, ops) in values {
    //         map.insert(node_type, ops);
    //     }

    //     map
    // }
}

impl<T: Clone + Default + 'static> Into<NodeBuilder<Op<T>>> for OpStore<NodeType, T> {
    fn into(self) -> NodeBuilder<Op<T>> {
        NodeBuilder::new(Arc::new(self))
    }
}
