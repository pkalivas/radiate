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

    pub fn valid_node_type(&self, node_type: NodeType) -> bool {
        match self.arity().unwrap_or(Arity::Any) {
            Arity::Zero => matches!(node_type, NodeType::Input | NodeType::Leaf),
            Arity::Any => matches!(
                node_type,
                NodeType::Output | NodeType::Root | NodeType::Vertex
            ),
            Arity::Exact(1) => matches!(node_type, NodeType::Edge | NodeType::Vertex),
            _ => matches!(node_type, NodeType::Vertex),
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

pub struct NodeStore<T> {
    values: Arc<RwLock<HashMap<NodeType, Vec<NodeValue<T>>>>>,
}

impl<T> NodeStore<T> {
    pub fn new() -> Self {
        NodeStore {
            values: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add(&self, values: Vec<T>)
    where
        T: Into<NodeValue<T>> + Clone,
    {
        let mut store_values = self.values.write().unwrap();

        for value in values {
            let node_value = value.into();
            let node_type = node_value.allowed_node_types();
            for node_type in node_type {
                store_values
                    .entry(node_type)
                    .or_insert_with(Vec::new)
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
        let all_values = values.values().collect::<Vec<&Vec<NodeValue<T>>>>();

        if all_values.is_empty() {
            return None;
        }

        let values = all_values
            .iter()
            .flat_map(|x| x.iter())
            .collect::<Vec<&NodeValue<T>>>();

        if values.is_empty() {
            return None;
        }

        Some(mapper(values))
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

#[cfg(test)]
mod tests {
    use crate::ops;

    use super::*;

    #[test]
    fn test_node_store() {
        let all_ops = ops::get_all_operations();
        let store = NodeStore::from(all_ops);

        store.insert(NodeType::Output, vec![Op::sigmoid()]);

        // store.add((0..3).map(Op::var).collect());
        // store.add(vec![Op::sigmoid(); 3]);
        // store.add(vec![Op::add(); 3]);
        // store.add(vec![Op::weight(); 3]);

        println!("{:?}", store);
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
