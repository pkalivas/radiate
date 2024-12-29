use radiate::{random_provider, Valid};

use super::{expr::Arity, GraphNode};
use std::{
    collections::{HashSet, VecDeque},
    fmt::Debug,
};

#[derive(Clone, PartialEq, Default)]
pub struct Graph<T> {
    nodes: Vec<GraphNode<T>>,
}

impl<T> Graph<T> {
    pub fn new(nodes: Vec<GraphNode<T>>) -> Self {
        Graph { nodes }
    }

    pub fn nodes(&self) -> &[GraphNode<T>] {
        &self.nodes
    }

    pub fn nodes_mut(&mut self) -> &mut [GraphNode<T>] {
        &mut self.nodes
    }

    pub fn add_node<C>(&mut self, cell: C) -> usize
    where
        C: Into<GraphNode<T>>,
    {
        let mut node = cell.into();
        let index = self.nodes.len();
        node.index = index;
        self.nodes.push(node);
        index
    }

    pub fn attach(&mut self, incoming: usize, outgoing: usize) {
        self.get_mut(incoming).outgoing_mut().insert(outgoing);
        self.get_mut(outgoing).incoming_mut().insert(incoming);
    }

    pub fn detach(&mut self, incoming: usize, outgoing: usize) {
        self.get_mut(incoming).outgoing_mut().remove(&outgoing);
        self.get_mut(outgoing).incoming_mut().remove(&incoming);
    }

    pub fn get(&self, index: usize) -> &GraphNode<T> {
        self.nodes.get(index).unwrap_or_else(|| {
            panic!(
                "Node index {} out of bounds for graph with {} nodes",
                index,
                self.nodes.len()
            )
        })
    }

    pub fn get_mut(&mut self, index: usize) -> &mut GraphNode<T> {
        let length = self.nodes.len();
        self.nodes.get_mut(index).unwrap_or_else(|| {
            panic!(
                "Node index {} out of bounds for graph with {} nodes",
                index, length
            )
        })
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
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

impl<T> From<Vec<GraphNode<T>>> for Graph<T> {
    fn from(nodes: Vec<GraphNode<T>>) -> Self {
        Graph { nodes }
    }
}

impl<T> Debug for Graph<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Graph {{\n")?;
        for node in self.nodes() {
            write!(f, "  {:?},\n", node)?;
        }
        write!(f, "}}")
    }
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

    if (source_node.outgoing.is_empty() || source_node.is_recurrent()) && !recurrent {
        return false;
    }

    let would_create_cycle = recurrent || !would_create_cycle(collection, source, target);

    would_create_cycle && source != target
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
    match node.cell.value.arity() {
        Arity::Zero => true,
        Arity::Nary(n) => n == node.incoming.len() as u8,
        Arity::Any => false,
    }
}
