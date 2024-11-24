use std::collections::VecDeque;

use super::*;
use crate::{Node, NodeCollection, NodeType, Tracer};

/// `GraphIterator` is an iterator that traverses a `Graph` in sudo-topological order. I say
/// "sudo-topological" because it is not a true topological order, but rather a topological order
/// that allows for recurrent connections. This iterator is used by the `GraphReducer` to evaluate
/// the nodes in a `Graph` in the correct order.
///
pub struct GraphIterator<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub graph: &'a Graph<T>,
    pub completed: Vec<bool>,
    pub index_queue: VecDeque<usize>,
    pub pending_index: usize,
}

impl<'a, T> GraphIterator<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(graph: &'a Graph<T>) -> Self {
        Self {
            graph,
            completed: vec![false; graph.len()],
            index_queue: VecDeque::new(),
            pending_index: 0,
        }
    }
}

impl<'a, T> Iterator for GraphIterator<'a, T>
where
    T: Clone + PartialEq + Default,
{
    type Item = &'a Node<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut min_pending_index = self.graph.len();
        for index in self.pending_index..self.graph.len() {
            if self.completed[index] {
                continue;
            }

            if let Some(node) = self.graph.get(index) {
                let mut degree = node.incoming.len();
                for incoming_index in &node.incoming {
                    if let Some(incoming_node) = self.graph.get(*incoming_index) {
                        if self.completed[incoming_node.index] || incoming_node.is_recurrent() {
                            degree -= 1;
                        }
                    }
                }

                if degree == 0 {
                    self.completed[node.index] = true;
                    self.index_queue.push_back(node.index);
                } else {
                    min_pending_index = std::cmp::min(min_pending_index, node.index);
                }
            }
        }

        self.pending_index = min_pending_index;

        if let Some(index) = self.index_queue.pop_front() {
            return match self.graph.get(index) {
                Some(node) => Some(node),
                None => None,
            };
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.graph.len()))
    }

    fn count(self) -> usize {
        self.graph.len()
    }
}

/// `GraphReducer` is a struct that is used to evaluate a `Graph` of `Node`s. It uses the `GraphIterator`
/// to traverse the `Graph` in a sudo-topological order and evaluate the nodes in the correct order.
///
/// On the first iteration it caches the order of nodes in the `Graph` and then uses that order to
/// evaluate the nodes in the correct order. This is a massive performance improvement.
///
pub struct GraphReducer<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub graph: &'a Graph<T>,
    pub tracers: Vec<Tracer<T>>,
    pub order: Vec<usize>,
    pub outputs: Vec<T>,
}

impl<'a, T> GraphReducer<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(graph: &'a Graph<T>) -> GraphReducer<'a, T> {
        let output_size = graph
            .iter()
            .filter(|node| node.node_type == NodeType::Output)
            .count();

        GraphReducer {
            graph,
            tracers: graph
                .iter()
                .map(|node| Tracer::new(GraphReducer::input_size(node)))
                .collect::<Vec<Tracer<T>>>(),
            order: Vec::with_capacity(graph.len()),
            outputs: vec![T::default(); output_size],
        }
    }

    #[inline]
    pub fn reduce(&mut self, inputs: &[T]) -> Vec<T> {
        if self.order.is_empty() {
            self.order = self
                .graph
                .topological_iter()
                .map(|node| node.index)
                .collect();
        }

        let mut output_index = 0;
        for index in &self.order {
            if let Some(node) = self.graph.get(*index) {
                if node.node_type == NodeType::Input {
                    self.tracers[node.index].add_input(inputs[node.index].clone());
                } else {
                    for incoming in &node.incoming {
                        let arg = self.tracers[*incoming]
                            .result
                            .clone()
                            .unwrap_or_else(|| T::default());
                        self.tracers[node.index].add_input(arg);
                    }
                }

                self.tracers[node.index].eval(&node);

                if node.node_type == NodeType::Output {
                    self.outputs[output_index] = self.tracers[node.index].result.clone().unwrap();
                    output_index += 1;
                }
            }
        }

        self.outputs.clone()
    }

    fn input_size(node: &Node<T>) -> usize {
        match node.node_type {
            NodeType::Input | NodeType::Link => 1,
            NodeType::Gate => node.value.arity() as usize,
            _ => node.incoming.len(),
        }
    }
}
