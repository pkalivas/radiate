use std::collections::VecDeque;

use crate::{Node, NodeCollection, NodeType, Tracer};

use super::{Graph, Tree};

pub struct BreadthFirstIterator<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub nodes: &'a [Node<T>],
    pub index: usize,
    pub queue: VecDeque<usize>,
}

impl<'a, T> BreadthFirstIterator<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(nodes: &'a [Node<T>], index: usize) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(index);

        Self {
            nodes,
            index,
            queue,
        }
    }
}

impl<'a, T> Iterator for BreadthFirstIterator<'a, T>
where
    T: Clone + PartialEq + Default,
{
    type Item = &'a Node<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(index) = self.queue.pop_front() {
            if let Some(node) = self.nodes.get(index) {
                for outgoing in &node.outgoing {
                    self.queue.push_back(*outgoing);
                }

                return Some(node);
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.nodes.len()))
    }

    fn count(self) -> usize {
        self.nodes.len()
    }
}

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

            let node = self.graph.get(index);
            let mut degree = node.incoming.len();
            for incoming_index in &node.incoming {
                let incoming_node = self.graph.get(*incoming_index);
                if self.completed[incoming_node.index] || incoming_node.is_recurrent() {
                    degree -= 1;
                }
            }

            if degree == 0 {
                self.completed[node.index] = true;
                self.index_queue.push_back(node.index);
            } else {
                min_pending_index = std::cmp::min(min_pending_index, node.index);
            }
        }

        self.pending_index = min_pending_index;

        if let Some(index) = self.index_queue.pop_front() {
            return Some(self.graph.get(index));
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
            let node = self.graph.get(*index);
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

        self.outputs.clone()
    }

    fn input_size(node: &Node<T>) -> usize {
        match node.node_type {
            NodeType::Input | NodeType::Link | NodeType::Leaf => 1,
            NodeType::Gate => node.value.arity() as usize,
            _ => node.incoming.len(),
        }
    }
}

pub struct TreeReducer<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub nodes: &'a Tree<T>,
    pub tracers: Vec<Tracer<T>>,
}

impl<'a, T> TreeReducer<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(nodes: &'a Tree<T>) -> TreeReducer<'a, T> {
        TreeReducer {
            nodes,
            tracers: nodes
                .iter()
                .map(|node| Tracer::new(TreeReducer::input_size(node)))
                .collect::<Vec<Tracer<T>>>(),
        }
    }

    #[inline]
    pub fn reduce(&mut self, inputs: &[T]) -> Vec<T> {
        self.eval_recurrent(0, inputs, &self.nodes.nodes)
    }

    fn eval_recurrent(&mut self, index: usize, input: &[T], nodes: &[Node<T>]) -> Vec<T> {
        let node = &nodes[index];

        if node.node_type == NodeType::Input || node.node_type == NodeType::Leaf {
            self.tracers[node.index].add_input(input[0].clone());
            self.tracers[node.index].eval(&node);
            return vec![self.tracers[node.index].result.clone().unwrap()];
        } else {
            for incoming in &node.outgoing {
                let arg = self.eval_recurrent(*incoming, input, nodes);
                self.tracers[node.index].add_input(arg[0].clone());
            }

            self.tracers[node.index].eval(&node);
            return vec![self.tracers[node.index].result.clone().unwrap()];
        }
    }

    fn input_size(node: &Node<T>) -> usize {
        match node.node_type {
            NodeType::Input | NodeType::Link | NodeType::Leaf => 1,
            NodeType::Gate => node.value.arity() as usize,
            _ => node.incoming.len(),
        }
    }
}
