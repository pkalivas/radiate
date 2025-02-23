use crate::collections::{Tree, TreeNode};
use std::collections::VecDeque;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collections::{Tree, TreeNode};

    use crate::Op;
    use crate::node::Node;
    use crate::ops::operation;

    #[test]
    fn test_tree_traversal() {
        // Create a simple tree:
        //       1
        //      / \
        //     2   3
        //    /
        //   4
        let leaf = Op::constant(4.0);
        let node2 = TreeNode::with_children(Op::constant(2.0), vec![TreeNode::new(leaf)]);

        let node3 = TreeNode::new(Op::constant(3.0));

        let root = Tree::new(TreeNode::with_children(
            Op::constant(1.0),
            vec![node2, node3],
        ));

        // Test pre-order
        let pre_order: Vec<f32> = root
            .iter_pre_order()
            .map(|n| match &n.value() {
                operation::Op::Const(_, v) => *v,
                _ => panic!("Expected constant but got {:?}", n.value()),
            })
            .collect();
        assert_eq!(pre_order, vec![1.0, 2.0, 4.0, 3.0]);

        // Test post-order
        let post_order: Vec<f32> = root
            .iter_post_order()
            .map(|n| match &n.value() {
                operation::Op::Const(_, v) => *v,
                _ => panic!("Expected constant"),
            })
            .collect();
        assert_eq!(post_order, vec![4.0, 2.0, 3.0, 1.0]);

        // Test breadth-first
        let bfs: Vec<f32> = root
            .iter_breadth_first()
            .map(|n| match &n.value() {
                operation::Op::Const(_, v) => *v,
                _ => panic!("Expected constant"),
            })
            .collect();
        assert_eq!(bfs, vec![1.0, 2.0, 3.0, 4.0]);
    }
}
