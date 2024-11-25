use std::collections::{HashSet, VecDeque};

use radiate::{engines::genome::genes::gene::Valid, RandomProvider};

use crate::{architects::node_collections::node::Node, NodeType};

use super::NodeFactory;

pub trait NodeRepairs<T>: Valid + Default + Clone
where
    T: Clone + PartialEq + Default,
{
    fn repair(&mut self, factory: Option<&NodeFactory<T>>) -> Self;
}

pub trait NodeCollection<T>: Valid + Default + Clone
where
    T: Clone + PartialEq + Default,
{
    fn from_nodes(nodes: Vec<Node<T>>) -> Self;

    fn get(&self, index: usize) -> Option<&Node<T>>;
    fn get_mut(&mut self, index: usize) -> Option<&mut Node<T>>;

    fn get_nodes(&self) -> &[Node<T>];
    fn get_nodes_mut(&mut self) -> &mut [Node<T>];

    fn set(&mut self, index: usize, node: Node<T>) -> &mut Self {
        self.get_nodes_mut()[index] = node;
        self
    }

    fn iter(&self) -> std::slice::Iter<Node<T>> {
        self.get_nodes().iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<Node<T>> {
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

pub fn reindex<T>(index: usize, nodes: &[&Node<T>]) -> Vec<Node<T>>
where
    T: Clone + PartialEq + Default,
{
    let mut new_nodes = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| Node {
            index: index + i,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
            ..(*node).clone()
        })
        .collect::<Vec<Node<T>>>();

    let ref_new_nodes = new_nodes.clone();

    let old_nodes = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.index, i))
        .collect::<std::collections::BTreeMap<usize, usize>>();

    for i in 0..new_nodes.len() {
        let old_node = nodes.get(i).unwrap();
        let new_node = &mut new_nodes[i];

        for incoming in old_node.incoming.iter() {
            if let Some(old_index) = old_nodes.get(incoming) {
                new_node
                    .incoming_mut()
                    .insert(ref_new_nodes[*old_index].index);
            }
        }

        for outgoing in old_node.outgoing.iter() {
            if let Some(old_index) = old_nodes.get(outgoing) {
                new_node
                    .outgoing_mut()
                    .insert(ref_new_nodes[*old_index].index);
            }
        }
    }

    new_nodes
}

#[inline]
pub fn get_cycles<T>(nodes: &[Node<T>], index: usize) -> Vec<usize>
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

#[inline]
pub fn can_connect<T>(collection: &[Node<T>], source: usize, target: usize, recurrent: bool) -> bool
where
    T: Clone + PartialEq + Default,
{
    let source_node = &collection.get(source).unwrap();
    let target_node = &collection.get(target).unwrap();

    if (source_node.outgoing.len() == 0 || source_node.is_recurrent()) && !recurrent {
        return false;
    }

    let would_create_cycle = recurrent || !would_create_cycle(collection, source, target);
    let nodes_are_weights =
        source_node.node_type == NodeType::Weight || target_node.node_type == NodeType::Weight;

    would_create_cycle && !nodes_are_weights && source != target
}

#[inline]
pub fn would_create_cycle<T>(collection: &[Node<T>], source: usize, target: usize) -> bool
where
    T: Clone + PartialEq + Default,
{
    let mut seen = HashSet::new();
    let mut visited = collection
        .get(target)
        .unwrap()
        .outgoing
        .iter()
        .collect::<Vec<&usize>>();

    while visited.len() != 0 {
        let node_index = visited.pop().unwrap();

        seen.insert(*node_index);

        if *node_index == source {
            return true;
        }

        for edge_index in collection
            .get(*node_index)
            .unwrap()
            .outgoing
            .iter()
            .filter(|edge_index| !seen.contains(edge_index))
        {
            visited.push(edge_index);
        }
    }

    false
}

pub fn is_locked<T>(node: &Node<T>) -> bool
where
    T: Clone + PartialEq + Default,
{
    if node.node_type == NodeType::Aggregate || node.node_type == NodeType::Output {
        return false;
    }

    node.incoming.len() == node.arity() as usize
}

#[inline]
pub fn random_source_node<T>(collection: &[Node<T>]) -> &Node<T>
where
    T: Clone + PartialEq + Default,
{
    random_node_of_type(
        collection,
        vec![
            NodeType::Input,
            NodeType::Gate,
            NodeType::Aggregate,
            NodeType::Link,
        ],
    )
}

#[inline]
pub fn random_target_node<T>(collection: &[Node<T>]) -> &Node<T>
where
    T: Clone + PartialEq + Default,
{
    random_node_of_type(collection, vec![NodeType::Output, NodeType::Aggregate])
}

#[inline]
fn random_node_of_type<T>(collection: &[Node<T>], node_types: Vec<NodeType>) -> &Node<T>
where
    T: Clone + PartialEq + Default,
{
    if node_types.len() == 0 {
        panic!("At least one node type must be specified.");
    }

    let gene_node_type_index = RandomProvider::random::<usize>() % node_types.len();
    let gene_node_type = node_types.get(gene_node_type_index).unwrap();

    let genes = match gene_node_type {
        NodeType::Input => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Input)
            .collect::<Vec<&Node<T>>>(),
        NodeType::Weight => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Weight)
            .collect::<Vec<&Node<T>>>(),
        NodeType::Gate => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Gate)
            .collect::<Vec<&Node<T>>>(),
        NodeType::Output => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Output)
            .collect::<Vec<&Node<T>>>(),
        NodeType::Link => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Link)
            .collect::<Vec<&Node<T>>>(),
        NodeType::Aggregate => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Aggregate)
            .collect::<Vec<&Node<T>>>(),
        NodeType::Root => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Root)
            .collect::<Vec<&Node<T>>>(),
        NodeType::Leaf => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Leaf)
            .collect::<Vec<&Node<T>>>(),
    };

    if genes.len() == 0 {
        return random_node_of_type(
            collection,
            node_types
                .iter()
                .filter(|nt| *nt != gene_node_type)
                .cloned()
                .collect(),
        );
    }

    let index = RandomProvider::random::<usize>() % genes.len();
    genes.get(index).unwrap()
}
