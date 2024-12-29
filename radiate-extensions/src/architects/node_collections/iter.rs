use std::collections::VecDeque;

use super::Graph;
use crate::node::Node;
use crate::{NodeCollection, Tree, TreeNode};

pub trait TreeIterator<T> {
    fn iter_pre_order(&self) -> PreOrderIterator<T>;
    fn iter_post_order(&self) -> PostOrderIterator<T>;
    fn iter_breadth_first(&self) -> TreeBreadthFirstIterator<T>;
}

impl<T> TreeIterator<T> for TreeNode<T> {
    fn iter_pre_order(&self) -> PreOrderIterator<T> {
        PreOrderIterator { stack: vec![self] }
    }

    fn iter_post_order(&self) -> PostOrderIterator<T> {
        PostOrderIterator {
            stack: vec![(self, false)],
        }
    }

    fn iter_breadth_first(&self) -> TreeBreadthFirstIterator<T> {
        TreeBreadthFirstIterator {
            queue: vec![self].into_iter().collect(),
        }
    }
}

impl<T> TreeIterator<T> for Tree<T> {
    fn iter_pre_order(&self) -> PreOrderIterator<T> {
        PreOrderIterator {
            stack: self
                .root()
                .map_or(Vec::new(), |root| vec![root].into_iter().collect()),
        }
    }

    fn iter_post_order(&self) -> PostOrderIterator<T> {
        PostOrderIterator {
            stack: self
                .root()
                .map_or(Vec::new(), |root| vec![(root, false)].into_iter().collect()),
        }
    }
    fn iter_breadth_first(&self) -> TreeBreadthFirstIterator<T> {
        TreeBreadthFirstIterator {
            queue: self
                .root()
                .map_or(VecDeque::new(), |root| vec![root].into_iter().collect()),
        }
    }
}

pub struct PreOrderIterator<'a, T> {
    stack: Vec<&'a TreeNode<T>>,
}

impl<'a, T> Iterator for PreOrderIterator<'a, T> {
    type Item = &'a TreeNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|node| {
            // Push children in reverse order for correct traversal
            if let Some(children) = node.children() {
                for child in children.iter().rev() {
                    self.stack.push(child);
                }
            }
            node
        })
    }
}

pub struct PostOrderIterator<'a, T> {
    stack: Vec<(&'a TreeNode<T>, bool)>,
}

impl<'a, T> Iterator for PostOrderIterator<'a, T> {
    type Item = &'a TreeNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, visited)) = self.stack.pop() {
            if visited {
                return Some(node);
            }
            self.stack.push((node, true));
            if let Some(children) = node.children() {
                for child in children.iter().rev() {
                    self.stack.push((child, false));
                }
            }
        }
        None
    }
}

pub struct TreeBreadthFirstIterator<'a, T> {
    queue: VecDeque<&'a TreeNode<T>>,
}

impl<'a, T> Iterator for TreeBreadthFirstIterator<'a, T> {
    type Item = &'a TreeNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.pop_front()?;
        if let Some(children) = node.children() {
            self.queue.extend(children.iter());
        }
        Some(node)
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{expr, Tree};

    #[test]
    fn test_tree_traversal() {
        // Create a simple tree:
        //       1
        //      / \
        //     2   3
        //    /
        //   4
        let leaf = expr::value(4.0);
        let node2 = TreeNode::with_children(expr::value(2.0), vec![TreeNode::new(leaf)]);

        let node3 = TreeNode::new(expr::value(3.0));

        let root = Tree::new(TreeNode::with_children(expr::add(), vec![node2, node3]));

        // Test pre-order
        let pre_order: Vec<f32> = root
            .iter_pre_order()
            .map(|n| match &n.value {
                expr::Operation::Const(_, v) => *v,
                _ => panic!("Expected constant"),
            })
            .collect();
        assert_eq!(pre_order, vec![1.0, 2.0, 4.0, 3.0]);

        // Test post-order
        let post_order: Vec<f32> = root
            .iter_post_order()
            .map(|n| match &n.value {
                expr::Operation::Const(_, v) => *v,
                _ => panic!("Expected constant"),
            })
            .collect();
        assert_eq!(post_order, vec![4.0, 2.0, 3.0, 1.0]);

        // Test breadth-first
        let bfs: Vec<f32> = root
            .iter_breadth_first()
            .map(|n| match &n.value {
                expr::Operation::Const(_, v) => *v,
                _ => panic!("Expected constant"),
            })
            .collect();
        assert_eq!(bfs, vec![1.0, 2.0, 3.0, 4.0]);
    }
}
