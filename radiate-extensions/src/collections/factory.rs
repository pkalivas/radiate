use crate::collections::{GraphNode, NodeType};
use crate::ops::operation::Op;
use crate::ops::{self};

use super::NodeCell;

use radiate::random_provider;
use std::collections::HashMap;

pub trait Factory<T> {
    type Input;
    fn new_instance(&self, input: Self::Input) -> T;
}

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

impl CellStore<Op<f32>> {
    pub fn regression(input_size: usize) -> CellStore<Op<f32>> {
        let inputs = (0..input_size).map(Op::var).collect::<Vec<Op<f32>>>();
        let mut store = CellStore::new();

        store.add_values(NodeType::Input, inputs.clone());
        store.add_values(NodeType::Vertex, ops::get_all_operations());
        store.add_values(NodeType::Edge, vec![Op::weight(), Op::identity()]);
        store.add_values(NodeType::Output, vec![Op::linear()]);

        store
    }
}

impl<C: NodeCell> Default for CellStore<C> {
    fn default() -> Self {
        CellStore::new()
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
            return match node_type {
                NodeType::Input => {
                    let value = random_provider::choose(values).new_instance();
                    GraphNode::new(index, node_type, value)
                }
                _ => {
                    let value = random_provider::choose(values);
                    GraphNode::new(index, node_type, value.new_instance())
                }
            };
        }

        GraphNode::new(index, node_type, C::default())
    }
}

// type CellStoreBlahhhh<C> = Arc<RwLock<HashMap<NodeType, Vec<C>>>>;

// #[derive(Default, Clone, Debug)]
// pub struct NodeFactory<C: NodeCell> {
//     pub node_values: CellStoreBlahhhh<C>,
// }

// impl<C: NodeCell> NodeFactory<C> {
//     pub fn new() -> Self {
//         NodeFactory {
//             node_values: Arc::new(RwLock::new(HashMap::new())),
//         }
//     }

//     pub fn inputs(self, values: Vec<C>) -> NodeFactory<C> {
//         self.add_node_values(NodeType::Input, values);
//         self
//     }

//     pub fn outputs(self, values: Vec<C>) -> NodeFactory<C> {
//         self.add_node_values(NodeType::Output, values);
//         self
//     }

//     pub fn vertices(self, values: Vec<C>) -> NodeFactory<C> {
//         self.add_node_values(NodeType::Vertex, values);
//         self
//     }

//     pub fn edges(self, values: Vec<C>) -> NodeFactory<C> {
//         self.add_node_values(NodeType::Edge, values);
//         self
//     }

//     pub fn add_node_values(&self, node_type: NodeType, values: Vec<C>) {
//         let mut read = self.node_values.write().unwrap();
//         read.insert(node_type, values);
//     }
// }

// impl NodeFactory<Op<f32>> {
//     pub fn regression(input_size: usize) -> NodeFactory<Op<f32>> {
//         let inputs = (0..input_size).map(Op::var).collect::<Vec<Op<f32>>>();
//         NodeFactory::new()
//             .inputs(inputs.clone())
//             .vertices(ops::get_all_operations())
//             .edges(vec![Op::weight(), Op::identity()])
//             .outputs(vec![Op::linear()])
//     }
// }

// impl<C: NodeCell + Clone + Default> Factory<GraphNode<C>> for HashMap<NodeType, Vec<C>> {
//     type Input = (usize, NodeType);

//     fn new_instance(&self, input: Self::Input) -> GraphNode<C> {
//         let (index, node_type) = input;
//         if let Some(values) = self.get(&node_type) {
//             return match node_type {
//                 NodeType::Input => {
//                     let value = random_provider::choose(values).new_instance();
//                     GraphNode::new(index, node_type, value)
//                 }
//                 _ => {
//                     let value = random_provider::choose(values);
//                     GraphNode::new(index, node_type, value.new_instance())
//                 }
//             };
//         }

//         GraphNode::new(index, node_type, C::default())
//     }
// }

// impl<C> Factory<GraphNode<C>> for NodeFactory<C>
// where
//     C: Clone + Default + NodeCell,
// {
//     type Input = (usize, NodeType);

//     fn new_instance(&self, input: Self::Input) -> GraphNode<C> {
//         let (index, node_type) = input;
//         let reader = self.node_values.read().unwrap();

//         if let Some(values) = reader.get(&node_type) {
//             return match node_type {
//                 NodeType::Input => {
//                     let value = values[index % values.len()].clone();
//                     GraphNode::new(index, node_type, value)
//                 }
//                 _ => {
//                     let value = random_provider::choose(values);
//                     GraphNode::new(index, node_type, value.new_instance())
//                 }
//             };
//         }

//         GraphNode::new(index, node_type, C::default())
//     }
// }
