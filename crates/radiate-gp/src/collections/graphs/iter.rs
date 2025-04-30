use crate::collections::GraphNode;
use std::collections::VecDeque;

/// [GraphIterator] is a trait that provides an iterator over any &[[GraphNode<T>]]. The iterator is used to
/// traverse the said nodes in a sudo-topological order.
pub trait GraphIterator<'a, T> {
    fn iter_topological(&'a self) -> GraphTopologicalIterator<'a, T>;
}

impl<'a, G: AsRef<[GraphNode<T>]>, T> GraphIterator<'a, T> for G {
    fn iter_topological(&'a self) -> GraphTopologicalIterator<'a, T> {
        GraphTopologicalIterator::new(self.as_ref())
    }
}

/// [GraphIterator] is an iterator that traverses a &[[GraphNode<T>]] in sudo-topological order. I say
/// "sudo-topological" because it is not a true topological order, but rather a topological order
/// that allows for recurrent connections.
pub struct GraphTopologicalIterator<'a, T> {
    graph: &'a [GraphNode<T>],
    completed: Vec<bool>,
    index_queue: VecDeque<usize>,
    pending_index: usize,
}

impl<'a, T> GraphTopologicalIterator<'a, T> {
    /// Create a new `GraphIterator` from a reference to a `Graph`.
    ///
    /// # Arguments
    /// - `graph`: A reference to the `Graph` to iterate over.
    pub fn new(graph: &'a [GraphNode<T>]) -> Self {
        GraphTopologicalIterator {
            graph,
            completed: vec![false; graph.len()],
            index_queue: VecDeque::new(),
            pending_index: 0,
        }
    }
}

/// Implement the `Iterator` trait for [GraphIterator].
/// The `Item` type is a reference to a [GraphNode].
///
/// This implementation is a bit more complex than the typical iterator implementation. The iterator
/// must traverse the graph in a sudo-topological order. This means that it must iterate over the
/// nodes in the graph in an order that respects the dependencies between the nodes. We
/// do this by keeping track of which nodes have been completed and which nodes are pending, it
/// then iterates over the nodes in the graph, checking the dependencies of each node to determine
/// if it can be completed. If a node can be completed, it is added to the index queue, which is
/// used to determine the order in which the nodes are returned by the iterator.
/// It is a 'sudo' topological order because it allows for recurrent connections in the graph.
impl<'a, T> Iterator for GraphTopologicalIterator<'a, T> {
    type Item = &'a GraphNode<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let mut min_pending_index = self.graph.len();
        for index in self.pending_index..self.graph.len() {
            if self.completed[index] {
                continue;
            }

            let node = &self.graph[index];
            let mut degree = node.incoming().len();
            for incoming_index in node.incoming() {
                let incoming_node = &self.graph[*incoming_index];
                if self.completed[incoming_node.index()] || incoming_node.is_recurrent() {
                    degree -= 1;
                }
            }

            if degree == 0 {
                self.completed[node.index()] = true;
                self.index_queue.push_back(node.index());
            } else {
                min_pending_index = std::cmp::min(min_pending_index, node.index());
            }
        }

        self.pending_index = min_pending_index;
        self.index_queue.pop_front().map(|idx| &self.graph[idx])
    }
}
