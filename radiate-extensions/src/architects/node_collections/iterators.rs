use std::collections::VecDeque;

use super::Graph;
use crate::node::Node;
use crate::NodeCollection;

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
