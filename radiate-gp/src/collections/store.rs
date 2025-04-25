use super::NodeType;
use crate::{Arity, Op};
use std::collections::HashMap;
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

    pub fn allowed_node_types(&self) -> Vec<NodeType> {
        match self.arity().unwrap_or(Arity::Any) {
            Arity::Zero => vec![NodeType::Input, NodeType::Leaf],
            Arity::Any => vec![NodeType::Output, NodeType::Root, NodeType::Vertex],
            Arity::Exact(1) => vec![NodeType::Edge, NodeType::Vertex],
            _ => vec![NodeType::Vertex],
        }
    }
}

macro_rules! impl_node_value {
    ($($t:ty),*) => {
        $(
            impl From<$t> for NodeValue<$t> {
                fn from(value: $t) -> Self {
                    NodeValue::Unbound(value)
                }
            }
        )*
    };
}

impl_node_value!(
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    f32,
    f64,
    String,
    bool,
    char,
    usize,
    isize,
    &'static str
);

#[derive(Default)]
pub struct NodeStore<T> {
    values: Arc<RwLock<HashMap<NodeType, Vec<NodeValue<T>>>>>,
}

impl<T> NodeStore<T> {
    pub fn new() -> Self {
        NodeStore {
            values: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn contains_type(&self, node_type: NodeType) -> bool {
        let values = self.values.read().unwrap();
        values.contains_key(&node_type)
            && values
                .get(&node_type)
                .is_some_and(|values| !values.is_empty())
    }

    pub fn add(&self, values: Vec<T>)
    where
        T: Into<NodeValue<T>> + Clone,
    {
        let mut store_values = self.values.write().unwrap();

        for value in values {
            let node_value = value.into();
            for node_type in node_value.allowed_node_types() {
                store_values
                    .entry(node_type)
                    .or_default()
                    .push(node_value.clone());
            }
        }
    }

    pub fn insert(&self, node_type: NodeType, values: Vec<T>)
    where
        T: Into<NodeValue<T>>,
    {
        let mut store_values = self.values.write().unwrap();
        store_values.insert(node_type, values.into_iter().map(|x| x.into()).collect());
    }

    pub fn map<F, K>(&self, mapper: F) -> Option<K>
    where
        F: Fn(Vec<&NodeValue<T>>) -> K,
    {
        let values = self.values.read().unwrap();
        let all_values = values
            .values()
            .flat_map(|val| val)
            .collect::<Vec<&NodeValue<T>>>();

        if all_values.is_empty() {
            return None;
        }

        Some(mapper(all_values))
    }

    pub fn map_by_type<F, K>(&self, node_type: NodeType, mapper: F) -> Option<K>
    where
        F: Fn(Vec<&NodeValue<T>>) -> K,
    {
        let values = self.values.read().unwrap();
        if let Some(values) = values.get(&node_type) {
            return Some(mapper(values.iter().collect()));
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
        store.add(values);
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
        for (node_type, values) in values.iter() {
            writeln!(f, "{node_type:?}:")?;
            for value in values {
                writeln!(f, "  {value:?}")?;
            }
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! node_store {
    ($($node_type:ident => $values:expr),+) => {
        {
            let store = NodeStore::new();
            $(
                store.insert(NodeType::$node_type, $values);
            )*
            store
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Factory, Node, ops};

    #[test]
    fn test_node_store() {
        let store = NodeStore::from(ops::all_ops());

        store.add((0..3).map(Op::var).collect());

        assert!(store.contains_type(NodeType::Input));
        assert!(store.contains_type(NodeType::Output));
        assert!(store.contains_type(NodeType::Edge));
        assert!(store.contains_type(NodeType::Vertex));
        assert!(store.contains_type(NodeType::Leaf));
        assert!(store.contains_type(NodeType::Root));
    }

    #[test]
    fn test_node_store_insert() {
        let store = NodeStore::new();
        let values = vec![1, 2, 3];
        store.insert(NodeType::Input, values.clone());

        assert!(store.contains_type(NodeType::Input));

        for value in values {
            assert!(
                store
                    .map_by_type(NodeType::Input, |values| {
                        values.iter().any(|v| v.value() == &value)
                    })
                    .unwrap_or(false)
            );
        }
    }

    #[test]
    fn test_node_store_macro() {
        let store = node_store! {
            Input => vec![1, 2, 3],
            Output => vec![4, 5, 6],
            Edge => vec![7, 8, 9],
            Vertex => vec![10, 11, 12]
        };

        assert!(store.contains_type(NodeType::Input));
        assert!(store.contains_type(NodeType::Output));
        assert!(store.contains_type(NodeType::Edge));
        assert!(store.contains_type(NodeType::Vertex));

        let graph_node = store.new_instance((2, NodeType::Vertex));

        assert_eq!(graph_node.index(), 2);
        assert_eq!(graph_node.node_type(), NodeType::Vertex);

        // hmmmm
        let tree_node = store.new_instance(NodeType::Vertex);
        assert_eq!(tree_node.node_type(), NodeType::Leaf);
        assert!(tree_node.is_leaf());
    }
}
