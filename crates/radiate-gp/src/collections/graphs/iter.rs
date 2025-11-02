use crate::collections::GraphNode;
use radiate_core::Valid;
use std::collections::VecDeque;

/// [GraphIterator] is a trait that provides an iterator over any &[[`GraphNode<T>`]]. The iterator is used to
/// traverse the said nodes in a pseudo-topological order.
pub trait GraphIterator<'a, T> {
    fn iter_topological(&'a self) -> GraphTopologicalIterator<'a, T>;
}

impl<'a, G: AsRef<[GraphNode<T>]>, T> GraphIterator<'a, T> for G {
    fn iter_topological(&'a self) -> GraphTopologicalIterator<'a, T> {
        GraphTopologicalIterator::new(self.as_ref())
    }
}

/// [GraphIterator] is an iterator that traverses a &[[`GraphNode<T>`]] in sudo-topological order. I say
/// "sudo-topological" because it is not a true topological order, but rather a topological order
/// that allows for recurrent connections.
pub struct GraphTopologicalIterator<'a, T> {
    graph: &'a [GraphNode<T>],
    completed: Vec<bool>,
    index_queue: VecDeque<usize>,
    pending_index: usize,
}

impl<'a, T> GraphTopologicalIterator<'a, T> {
    /// Create a new `GraphIterator` from a reference to a [`GraphNode<T>`].
    ///
    /// # Arguments
    /// - `graph`: A reference to the `Graph` to iterate over.
    pub fn new(graph: &'a [GraphNode<T>]) -> Self {
        let is_valid = !graph.iter().any(|node| !node.is_valid());
        GraphTopologicalIterator {
            graph,
            completed: vec![false; graph.len()],
            index_queue: VecDeque::new(),
            pending_index: if is_valid { 0 } else { graph.len() },
        }
    }
}

/// Implement the `Iterator` trait for [GraphTopologicalIterator].
/// The `Item` type is a reference to a [GraphNode].
///
/// This implementation is a bit more complex than the typical iterator implementation. The iterator
/// must traverse the graph in a pseudo-topological order. This means that it must iterate over the
/// nodes in the graph in an order that respects the dependencies between the nodes. We
/// do this by keeping track of which nodes have been completed and which nodes are pending, it
/// then iterates over the nodes in the graph, checking the dependencies of each node to determine
/// if it can be completed. If a node can be completed, it is added to the index queue, which is
/// used to determine the order in which the nodes are returned by the iterator.
/// It is a 'pseudo' topological order because it allows for recurrent connections in the graph.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::{Graph, GraphNode, NodeType};
    use crate::ops::Op;

    #[test]
    fn test_graph_iterator() {
        let graph = Graph::new(vec![
            GraphNode::from((0, NodeType::Input, Op::var(0))).with_outgoing([2]),
            GraphNode::from((1, NodeType::Input, Op::var(1))).with_outgoing([2]),
            GraphNode::from((2, NodeType::Vertex, Op::add()))
                .with_incoming([0, 1])
                .with_outgoing([3]),
            GraphNode::from((3, NodeType::Output, Op::linear())).with_incoming([2]),
        ]);

        let mut iter = graph.iter_topological();

        assert_eq!(iter.next().unwrap().index(), 0);
        assert_eq!(iter.next().unwrap().index(), 1);
        assert_eq!(iter.next().unwrap().index(), 2);
        assert_eq!(iter.next().unwrap().index(), 3);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_graph_iterator_recurrent() {
        let nodes = vec![
            GraphNode::from((0, NodeType::Input, Op::var(0), vec![], vec![2])),
            GraphNode::from((1, NodeType::Input, Op::var(1), vec![], vec![2])),
            GraphNode::from((2, NodeType::Vertex, Op::add(), vec![0, 1], vec![3])),
            GraphNode::from((3, NodeType::Vertex, Op::mul(), vec![2], vec![2])),
            GraphNode::from((4, NodeType::Output, Op::linear(), vec![3], vec![])),
        ];

        let graph = Graph::new(nodes);
        let mut iter = graph.iter_topological();

        assert_eq!(iter.next().unwrap().index(), 0);
        assert_eq!(iter.next().unwrap().index(), 1);
        assert_eq!(iter.next().unwrap().index(), 2);
        assert_eq!(iter.next().unwrap().index(), 3);
        assert_eq!(iter.next().unwrap().index(), 4);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_graph_iterator_disconnected() {
        let nodes = vec![
            GraphNode::from((0, NodeType::Input, Op::var(0))).with_outgoing([2]),
            GraphNode::from((1, NodeType::Input, Op::var(1))),
            GraphNode::from((2, NodeType::Vertex, Op::add())).with_incoming([0]),
            GraphNode::from((3, NodeType::Output, Op::linear())).with_incoming([2]),
        ];

        let results = Graph::new(nodes)
            .iter_topological()
            .map(|node| node.index())
            .collect::<Vec<usize>>();

        assert!(results.is_empty());
    }

    #[test]
    fn test_graph_deep_cycles() {
        let mut graph = Graph::<Op<f32>>::default();

        graph.insert(NodeType::Input, Op::var(0));
        graph.insert(NodeType::Vertex, Op::diff());
        graph.insert(NodeType::Output, Op::sigmoid());
        graph.insert(NodeType::Vertex, Op::div());
        graph.insert(NodeType::Vertex, Op::pow());
        graph.insert(NodeType::Edge, Op::weight());
        graph.insert(NodeType::Edge, Op::identity());
        graph.insert(NodeType::Vertex, Op::exp());
        graph.insert(NodeType::Vertex, Op::cos());
        graph.insert(NodeType::Edge, Op::weight());

        graph.attach(0, 1);
        graph.attach(1, 1);
        graph.attach(4, 1);
        graph.attach(7, 1);
        graph.attach(1, 2);
        graph.attach(3, 2);
        graph.attach(9, 2);
        graph.attach(0, 3);
        graph.attach(5, 3);
        graph.attach(0, 4);
        graph.attach(8, 4);
        graph.attach(1, 5);
        graph.attach(3, 6);
        graph.attach(4, 7);
        graph.attach(6, 8);
        graph.attach(7, 9);

        graph.set_cycles(vec![]);

        let results = graph
            .iter_topological()
            .map(|node| node.index())
            .collect::<Vec<usize>>();

        assert_eq!(results, vec![0, 1, 3, 4, 5, 6, 7, 8, 9, 2]);
    }
}
