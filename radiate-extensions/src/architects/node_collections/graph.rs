use std::collections::{HashSet, VecDeque};
use std::ops::{Index, IndexMut};

use radiate::{random_provider, Chromosome, Valid};

use super::operation::Arity;
use super::GraphIterator;
use crate::node::GraphNode;
use crate::{Direction, NodeType};

#[derive(Clone, PartialEq, Default)]
pub struct Graph<T> {
    pub nodes: Vec<GraphNode<T>>,
}

impl<T> Graph<T> {
    pub fn new(nodes: Vec<GraphNode<T>>) -> Self {
        Graph { nodes }
    }

    pub fn iter(&self) -> impl Iterator<Item = &GraphNode<T>> {
        self.nodes.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut GraphNode<T>> {
        self.nodes.iter_mut()
    }

    pub fn topological_iter(&self) -> impl Iterator<Item = &GraphNode<T>> {
        GraphIterator::new(self)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn get_mut(&mut self, index: usize) -> &mut GraphNode<T> {
        self.nodes.get_mut(index).unwrap()
    }

    pub fn get(&self, index: usize) -> &GraphNode<T> {
        self.nodes.get(index).unwrap()
    }

    pub fn attach(&mut self, incoming: usize, outgoing: usize) -> &mut Self {
        self.as_mut()[incoming].outgoing_mut().insert(outgoing);
        self.as_mut()[outgoing].incoming_mut().insert(incoming);
        self
    }

    pub fn detach(&mut self, incoming: usize, outgoing: usize) -> &mut Self {
        self.as_mut()[incoming].outgoing_mut().remove(&outgoing);
        self.as_mut()[outgoing].incoming_mut().remove(&incoming);
        self
    }

    pub fn set_cycles(mut self, indecies: Vec<usize>) -> Graph<T> {
        if indecies.is_empty() {
            let all_indices = self
                .as_ref()
                .iter()
                .map(|node| node.index)
                .collect::<Vec<usize>>();

            return self.set_cycles(all_indices);
        }

        for idx in indecies {
            let node_cycles = get_cycles(self.as_ref(), idx);

            if node_cycles.is_empty() {
                let node = self.get_mut(idx);
                node.direction = Direction::Forward;
            } else {
                for cycle_idx in node_cycles {
                    let node = self.get_mut(cycle_idx);
                    node.direction = Direction::Backward;
                }
            }
        }

        self
    }
}

impl<T> Chromosome for Graph<T>
where
    T: Clone + PartialEq + Default,
{
    type Gene = GraphNode<T>;

    fn from_genes(genes: Vec<GraphNode<T>>) -> Self {
        Graph { nodes: genes }
    }

    fn get_genes(&self) -> &[GraphNode<T>] {
        &self.nodes
    }

    fn get_genes_mut(&mut self) -> &mut [GraphNode<T>] {
        &mut self.nodes
    }
}

impl<T> AsRef<[GraphNode<T>]> for Graph<T> {
    fn as_ref(&self) -> &[GraphNode<T>] {
        &self.nodes
    }
}

impl<T> AsMut<[GraphNode<T>]> for Graph<T> {
    fn as_mut(&mut self) -> &mut [GraphNode<T>] {
        &mut self.nodes
    }
}

impl<T> Index<usize> for Graph<T> {
    type Output = GraphNode<T>;

    fn index(&self, index: usize) -> &Self::Output {
        self.nodes.get(index).expect("Index out of bounds.")
    }
}

impl<T> IndexMut<usize> for Graph<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.nodes.get_mut(index).expect("Index out of bounds.")
    }
}

impl<T> Valid for Graph<T>
where
    T: Clone + PartialEq + Default,
{
    fn is_valid(&self) -> bool {
        self.nodes.iter().all(|node| node.is_valid())
    }
}

impl<T> IntoIterator for Graph<T>
where
    T: Clone + PartialEq + Default,
{
    type Item = GraphNode<T>;
    type IntoIter = std::vec::IntoIter<GraphNode<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
    }
}

impl<T> std::fmt::Debug for Graph<T>
where
    T: Clone + PartialEq + Default + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Graph {{\n")?;
        for node in self.as_ref() {
            write!(f, "  {:?},\n", node)?;
        }
        write!(f, "}}")
    }
}

#[inline]
pub fn reindex<T>(index: usize, nodes: &[&GraphNode<T>]) -> Vec<GraphNode<T>>
where
    T: Clone,
{
    let mut new_nodes = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| GraphNode {
            index: index + i,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
            ..(*node).clone()
        })
        .collect::<Vec<GraphNode<T>>>();

    let ref_new_nodes = new_nodes.clone();

    let old_nodes = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.index, i))
        .collect::<std::collections::BTreeMap<usize, usize>>();

    for i in 0..nodes.len() {
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
pub fn get_cycles<T>(nodes: &[GraphNode<T>], index: usize) -> Vec<usize> {
    let mut path = Vec::new();
    let mut seen = HashSet::new();
    let mut current = nodes[index]
        .incoming()
        .iter()
        .cloned()
        .collect::<VecDeque<usize>>();

    while !current.is_empty() {
        let current_index = current.pop_front().unwrap();
        let current_node = &nodes[current_index];

        if seen.contains(&current_index) {
            continue;
        }

        if current_index == index {
            return path;
        }

        seen.insert(current_index);

        if !current_node.incoming().is_empty() {
            path.push(current_index);
            for outgoing in current_node.incoming().iter() {
                current.push_back(*outgoing);
            }
        }
    }

    Vec::new()
}

#[inline]
pub fn can_connect<T>(
    collection: &[GraphNode<T>],
    source: usize,
    target: usize,
    recurrent: bool,
) -> bool {
    let source_node = &collection.get(source).unwrap();
    let target_node = &collection.get(target).unwrap();

    if (source_node.outgoing.is_empty() || source_node.is_recurrent()) && !recurrent {
        return false;
    }

    let would_create_cycle = recurrent || !would_create_cycle(collection, source, target);
    let nodes_are_weights =
        source_node.node_type == NodeType::Edge || target_node.node_type == NodeType::Edge;

    would_create_cycle && !nodes_are_weights && source != target
}

#[inline]
pub fn would_create_cycle<T>(collection: &[GraphNode<T>], source: usize, target: usize) -> bool {
    let mut seen = HashSet::new();
    let mut visited = collection
        .get(target)
        .unwrap()
        .outgoing
        .iter()
        .collect::<Vec<&usize>>();

    while !visited.is_empty() {
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

pub fn is_locked<T>(node: &GraphNode<T>) -> bool {
    if node.value.arity() == Arity::Any {
        return false;
    }

    node.incoming.len() == *node.value.arity() as usize
}

#[inline]
pub fn random_source_node<T>(collection: &[GraphNode<T>]) -> &GraphNode<T> {
    random_node_of_type(collection, vec![NodeType::Input, NodeType::Vertex])
}

#[inline]
pub fn random_target_node<T>(collection: &[GraphNode<T>]) -> &GraphNode<T> {
    random_node_of_type(collection, vec![NodeType::Output, NodeType::Vertex])
}

#[inline]
fn random_node_of_type<T>(collection: &[GraphNode<T>], node_types: Vec<NodeType>) -> &GraphNode<T> {
    if node_types.is_empty() {
        panic!("At least one node type must be specified.");
    }

    let gene_node_type_index = random_provider::random::<usize>() % node_types.len();
    let gene_node_type = node_types.get(gene_node_type_index).unwrap();

    let genes = match gene_node_type {
        NodeType::Input => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Input)
            .collect::<Vec<&GraphNode<T>>>(),
        NodeType::Output => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Output)
            .collect::<Vec<&GraphNode<T>>>(),
        NodeType::Vertex => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Vertex)
            .collect::<Vec<&GraphNode<T>>>(),
        NodeType::Edge => collection
            .iter()
            .filter(|node| node.node_type == NodeType::Edge)
            .collect::<Vec<&GraphNode<T>>>(),
    };

    if genes.is_empty() {
        return random_node_of_type(
            collection,
            node_types
                .iter()
                .filter(|nt| *nt != gene_node_type)
                .cloned()
                .collect(),
        );
    }

    let index = random_provider::random::<usize>() % genes.len();
    genes.get(index).unwrap()
}
