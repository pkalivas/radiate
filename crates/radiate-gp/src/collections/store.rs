use super::NodeType;
use crate::{Arity, Node, Op, TreeIterator, TreeNode};
#[cfg(feature = "serde")]
use serde::{
    Deserialize, Serialize,
    de::Deserializer,
    ser::{Error as SerError, Serializer},
};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    values: Arc<RwLock<BTreeMap<NodeType, Vec<NodeValue<T>>>>>,
}

impl<T> NodeStore<T> {
    pub fn new() -> Self {
        NodeStore {
            values: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    pub fn ref_count(&self) -> usize {
        Arc::strong_count(&self.values)
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

    pub fn merge(&self, other: impl Into<NodeStore<T>>)
    where
        T: Clone,
    {
        let other_store = other.into();
        let mut store_values = self.values.write().unwrap();
        let other_values = other_store.values.read().unwrap();

        for (node_type, values) in other_values.iter() {
            store_values
                .entry(*node_type)
                .or_default()
                .extend(values.iter().cloned());
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
        F: Fn(&[NodeValue<T>]) -> K,
    {
        let values = self.values.read().unwrap();
        if let Some(values) = values.get(&node_type) {
            return Some(mapper(values));
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
    T: Into<NodeValue<T>> + Clone,
{
    fn from(values: Vec<(NodeType, Vec<T>)>) -> Self {
        let store = NodeStore::new();
        for (node_type, ops) in values {
            store.insert(node_type, ops);
        }

        if !store.contains_type(NodeType::Leaf) && store.contains_type(NodeType::Input) {
            let input_values = store
                .map(|vals| {
                    vals.iter()
                        .filter_map(|v| match v.arity() {
                            Some(Arity::Zero) => Some((*v.value()).clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            store.insert(NodeType::Leaf, input_values);
        }

        if !store.contains_type(NodeType::Root) && store.contains_type(NodeType::Output) {
            let output_values = store
                .map(|vals| {
                    vals.iter()
                        .filter_map(|v| match v.arity() {
                            Some(Arity::Any) | Some(Arity::Exact(_)) => Some((*v.value()).clone()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            store.insert(NodeType::Root, output_values);
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

impl<T> From<Vec<TreeNode<Op<T>>>> for NodeStore<Op<T>>
where
    T: Clone,
{
    fn from(tree_nodes: Vec<TreeNode<Op<T>>>) -> Self {
        let store = NodeStore::new();
        for root in tree_nodes.iter() {
            store.merge(NodeStore::from(root.clone()));
        }

        store
    }
}

impl<T> From<TreeNode<Op<T>>> for NodeStore<Op<T>>
where
    T: Clone,
{
    fn from(tree_node: TreeNode<Op<T>>) -> Self {
        let store = NodeStore::new();

        let mut nodes = Vec::new();
        tree_node.iter_pre_order().for_each(|node| {
            #[cfg(feature = "pgm")]
            {
                if let Op::PGM(name, arity, programs, eval_fn) = node.value() {
                    nodes.push(Op::PGM(
                        name,
                        *arity,
                        Arc::new(programs.iter().map(|p| p.clone()).collect()),
                        *eval_fn,
                    ));

                    for prog in programs.iter() {
                        nodes.extend(prog.iter_pre_order().map(|n| n.value().clone()));
                    }
                } else {
                    nodes.push(node.value().clone());
                }
            }
            #[cfg(not(feature = "pgm"))]
            {
                nodes.push(node.value().clone());
            }
        });

        store.add(nodes);
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

#[cfg(feature = "serde")]
impl<T> Serialize for NodeStore<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Read the values from the RwLock
        let values = self
            .values
            .read()
            .map_err(|_| S::Error::custom("Failed to acquire read lock"))?;

        // Convert the HashMap into a serializable format
        let serializable: Vec<_> = values
            .iter()
            .map(|(node_type, values)| (node_type, values))
            .collect();

        serializable.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T> Deserialize<'de> for NodeStore<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let values: Vec<(NodeType, Vec<NodeValue<T>>)> = Vec::deserialize(deserializer)?;

        let mut map = BTreeMap::new();
        for (node_type, node_values) in values {
            map.insert(node_type, node_values);
        }

        Ok(NodeStore {
            values: Arc::new(RwLock::new(map)),
        })
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

    fn create_test_store() -> NodeStore<i32> {
        let store = NodeStore::new();

        store.insert(NodeType::Input, vec![1, 2, 3]);
        store.insert(NodeType::Output, vec![4, 5]);
        store.insert(NodeType::Vertex, vec![6, 7, 8, 9]);

        let bounded_values = vec![
            NodeValue::Bounded(10, Arity::Exact(2)),
            NodeValue::Bounded(11, Arity::Zero),
        ];

        store.insert(
            NodeType::Edge,
            bounded_values
                .into_iter()
                .map(|v| v.value().clone())
                .collect(),
        );

        store
    }

    #[test]
    fn test_node_store() {
        let store = NodeStore::from(ops::all_ops());

        store.add(Op::vars(0..3));

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

    #[test]
    fn test_insert_and_contains() {
        let store = NodeStore::new();

        store.insert(NodeType::Input, vec![1, 2, 3]);
        assert!(store.contains_type(NodeType::Input));

        store.insert(NodeType::Output, vec![4, 5]);
        assert!(store.contains_type(NodeType::Output));

        assert!(!store.contains_type(NodeType::Vertex));
    }

    #[test]
    fn test_new_store_is_empty() {
        let store: NodeStore<i32> = NodeStore::new();
        assert!(!store.contains_type(NodeType::Input));
        assert!(!store.contains_type(NodeType::Output));
        assert!(!store.contains_type(NodeType::Vertex));
    }

    #[test]
    fn test_map_operation() {
        let store = NodeStore::new();
        store.insert(NodeType::Input, vec![1, 2, 3]);
        store.insert(NodeType::Output, vec![4, 5]);

        // Test map operation that counts total values
        let total = store.map(|values| values.len()).unwrap();
        assert_eq!(total, 5);

        // Test map operation that sums all values
        let sum: i32 = store
            .map(|values| values.iter().map(|v| v.value()).sum())
            .unwrap();
        assert_eq!(sum, 15);
    }

    #[test]
    fn test_map_by_type() {
        let store = NodeStore::new();
        store.insert(NodeType::Input, vec![1, 2, 3]);
        store.insert(NodeType::Output, vec![4, 5]);

        // Test map_by_type for Input
        let input_sum: i32 = store
            .map_by_type(NodeType::Input, |values| {
                values.iter().map(|v| v.value()).sum()
            })
            .unwrap();
        assert_eq!(input_sum, 6);

        // Test map_by_type for Output
        let output_sum: i32 = store
            .map_by_type(NodeType::Output, |values| {
                values.iter().map(|v| v.value()).sum()
            })
            .unwrap();
        assert_eq!(output_sum, 9);

        // Test map_by_type for non-existent type
        let result = store.map_by_type(NodeType::Vertex, |values| values.len());
        assert!(result.is_none());
    }

    #[test]
    fn test_from_hashmap() {
        let mut map = HashMap::new();
        map.insert(NodeType::Input, vec![1, 2, 3]);
        map.insert(NodeType::Output, vec![4, 5]);

        let store: NodeStore<i32> = map.into();

        assert!(store.contains_type(NodeType::Input));
        assert!(store.contains_type(NodeType::Output));
        assert!(!store.contains_type(NodeType::Vertex));
    }

    #[test]
    fn test_from_vec_of_tuples() {
        let values = vec![
            (NodeType::Input, vec![1, 2, 3]),
            (NodeType::Output, vec![4, 5]),
        ];

        let store: NodeStore<i32> = values.into();

        assert!(store.contains_type(NodeType::Input));
        assert!(store.contains_type(NodeType::Output));
        assert!(!store.contains_type(NodeType::Vertex));
    }

    #[test]
    fn test_empty_map_returns_none() {
        let store: NodeStore<i32> = NodeStore::new();

        // map should return None for empty store
        assert!(store.map(|_| 42).is_none());

        // map_by_type should return None for empty store
        assert!(store.map_by_type(NodeType::Input, |_| 42).is_none());
    }

    #[test]
    fn test_insert_overwrites_existing() {
        let store = NodeStore::new();

        // First insert
        store.insert(NodeType::Input, vec![1, 2, 3]);

        // Second insert should overwrite
        store.insert(NodeType::Input, vec![4, 5]);

        // Verify only new values exist
        let values: Vec<i32> = store
            .map_by_type(NodeType::Input, |values| {
                values.iter().map(|v| v.value().clone()).collect()
            })
            .unwrap();

        assert_eq!(values, vec![4, 5]);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_basic() {
        let store = create_test_store();

        // Serialize to JSON
        let serialized = serde_json::to_string(&store).unwrap();

        // Deserialize back
        let deserialized: NodeStore<i32> = serde_json::from_str(&serialized).unwrap();

        // Verify the contents
        assert_eq!(store, deserialized);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_empty() {
        let store: NodeStore<i32> = NodeStore::new();

        let serialized = serde_json::to_string(&store).unwrap();
        let deserialized: NodeStore<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(store, deserialized);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_with_bounded_values() {
        let store = NodeStore::new();

        // Create a store with only bounded values
        let bounded_values = vec![
            NodeValue::Bounded(1, Arity::Exact(2)),
            NodeValue::Bounded(2, Arity::Zero),
            NodeValue::Bounded(3, Arity::Any),
        ];

        store.insert(
            NodeType::Vertex,
            bounded_values
                .into_iter()
                .map(|v| v.value().clone())
                .collect(),
        );

        let serialized = serde_json::to_string(&store).unwrap();
        let deserialized: NodeStore<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(store, deserialized);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_with_unbound_values() {
        let store = NodeStore::new();

        // Create a store with only unbound values
        let unbound_values = vec![1, 2, 3, 4, 5];
        store.insert(NodeType::Vertex, unbound_values);

        let serialized = serde_json::to_string(&store).unwrap();
        let deserialized: NodeStore<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(store, deserialized);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_mixed_values() {
        let store = NodeStore::new();

        // Mix of bounded and unbound values
        let mixed_values = vec![
            NodeValue::Bounded(1, Arity::Exact(2)),
            NodeValue::Unbound(2),
            NodeValue::Bounded(3, Arity::Zero),
            NodeValue::Unbound(4),
        ];

        store.insert(
            NodeType::Vertex,
            mixed_values
                .into_iter()
                .map(|v| v.value().clone())
                .collect(),
        );

        let serialized = serde_json::to_string(&store).unwrap();
        let deserialized: NodeStore<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(store, deserialized);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_all_node_types() {
        let store = NodeStore::new();

        // Add values for all node types
        store.insert(NodeType::Input, vec![1, 2]);
        store.insert(NodeType::Output, vec![3, 4]);
        store.insert(NodeType::Vertex, vec![5, 6]);
        store.insert(NodeType::Edge, vec![7, 8]);
        store.insert(NodeType::Leaf, vec![9, 10]);
        store.insert(NodeType::Root, vec![11, 12]);

        let serialized = serde_json::to_string(&store).unwrap();
        let deserialized: NodeStore<i32> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(store, deserialized);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serialize_deserialize_complex_type() {
        // Test with a more complex type (String)
        let store = NodeStore::new();

        let values = vec![
            NodeValue::Bounded("hello".to_string(), Arity::Exact(2)),
            NodeValue::Unbound("world".to_string()),
            NodeValue::Bounded("test".to_string(), Arity::Zero),
        ];

        store.insert(
            NodeType::Vertex,
            values.into_iter().map(|v| v.value().clone()).collect(),
        );

        let serialized = serde_json::to_string(&store).unwrap();
        let deserialized: NodeStore<String> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(store, deserialized);
    }
}
