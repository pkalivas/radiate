use std::collections::{HashSet, VecDeque};

use radiate::engines::genome::genes::gene::Valid;

use crate::architects::node_collections::graph_node::GraphNode;
use crate::NodeType;

pub trait Node: Valid {
    type Value;

    fn node_type(&self) -> &NodeType;
    fn value(&self) -> &Self::Value;
}

pub trait NodeCollection<C, N, T>: Valid + Default + Clone
where
    C: NodeCollection<C, N, T>,
    N: Node,
    T: Clone + PartialEq + Default,
{
    fn from_nodes(nodes: Vec<GraphNode<T>>) -> Self;

    fn get(&self, index: usize) -> Option<&GraphNode<T>>;
    fn get_mut(&mut self, index: usize) -> Option<&mut GraphNode<T>>;

    fn get_nodes(&self) -> &[GraphNode<T>];
    fn get_nodes_mut(&mut self) -> &mut [GraphNode<T>];

    fn set_cycles(self, indecies: Vec<usize>) -> C;

    fn add(&mut self, nodes: Vec<GraphNode<T>>);

    fn set(&mut self, index: usize, node: GraphNode<T>) -> &mut Self {
        self.get_nodes_mut()[index] = node;
        self
    }

    fn iter(&self) -> std::slice::Iter<GraphNode<T>> {
        self.get_nodes().iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<GraphNode<T>> {
        self.get_nodes_mut().iter_mut()
    }

    fn len(&self) -> usize {
        self.get_nodes().len()
    }

    fn attach(&mut self, incoming: usize, outgoing: usize) -> &mut Self {
        self.get_nodes_mut()[incoming]
            .outgoing_mut()
            .insert(outgoing);
        self.get_nodes_mut()[outgoing]
            .incoming_mut()
            .insert(incoming);
        self
    }

    fn detach(&mut self, incoming: usize, outgoing: usize) -> &mut Self {
        self.get_nodes_mut()[incoming]
            .outgoing_mut()
            .remove(&outgoing);
        self.get_nodes_mut()[outgoing]
            .incoming_mut()
            .remove(&incoming);
        self
    }
}

pub fn get_cycles<T>(nodes: &[GraphNode<T>], index: usize) -> Vec<usize>
where
    T: Clone + PartialEq + Default,
{
    let mut path = Vec::new();
    let mut seen = HashSet::new();
    let mut current = nodes[index]
        .incoming()
        .iter()
        .cloned()
        .collect::<VecDeque<usize>>();

    while current.len() > 0 {
        let current_index = current.pop_front().unwrap();
        let current_node = &nodes[current_index];

        if seen.contains(&current_index) {
            continue;
        }

        if current_index == index {
            return path;
        }

        seen.insert(current_index);

        if current_node.incoming().len() != 0 {
            path.push(current_index);
            for outgoing in current_node.incoming().iter() {
                current.push_back(*outgoing);
            }
        }
    }

    Vec::new()
}
