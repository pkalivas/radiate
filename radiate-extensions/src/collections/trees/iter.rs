use crate::{
    collections::{Tree, TreeNode},
    NodeCell,
};
use std::collections::VecDeque;

pub trait TreeIterator<C: NodeCell> {
    fn iter_pre_order(&self) -> PreOrderIterator<C>;
    fn iter_post_order(&self) -> PostOrderIterator<C>;
    fn iter_breadth_first(&self) -> TreeBreadthFirstIterator<C>;
}

impl<C: NodeCell> TreeIterator<C> for TreeNode<C> {
    fn iter_pre_order(&self) -> PreOrderIterator<C> {
        PreOrderIterator { stack: vec![self] }
    }

    fn iter_post_order(&self) -> PostOrderIterator<C> {
        PostOrderIterator {
            stack: vec![(self, false)],
        }
    }

    fn iter_breadth_first(&self) -> TreeBreadthFirstIterator<C> {
        TreeBreadthFirstIterator {
            queue: vec![self].into_iter().collect(),
        }
    }
}

impl<C: NodeCell> TreeIterator<C> for Tree<C> {
    fn iter_pre_order(&self) -> PreOrderIterator<C> {
        PreOrderIterator {
            stack: self
                .root()
                .map_or(Vec::new(), |root| vec![root].into_iter().collect()),
        }
    }

    fn iter_post_order(&self) -> PostOrderIterator<C> {
        PostOrderIterator {
            stack: self
                .root()
                .map_or(Vec::new(), |root| vec![(root, false)].into_iter().collect()),
        }
    }
    fn iter_breadth_first(&self) -> TreeBreadthFirstIterator<C> {
        TreeBreadthFirstIterator {
            queue: self
                .root()
                .map_or(VecDeque::new(), |root| vec![root].into_iter().collect()),
        }
    }
}

pub struct PreOrderIterator<'a, C: NodeCell> {
    stack: Vec<&'a TreeNode<C>>,
}

impl<'a, C: NodeCell> Iterator for PreOrderIterator<'a, C> {
    type Item = &'a TreeNode<C>;

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

pub struct PostOrderIterator<'a, C: NodeCell> {
    stack: Vec<(&'a TreeNode<C>, bool)>,
}

impl<'a, C: NodeCell> Iterator for PostOrderIterator<'a, C> {
    type Item = &'a TreeNode<C>;

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

pub struct TreeBreadthFirstIterator<'a, C: NodeCell> {
    queue: VecDeque<&'a TreeNode<C>>,
}

impl<'a, C: NodeCell> Iterator for TreeBreadthFirstIterator<'a, C> {
    type Item = &'a TreeNode<C>;

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

    use crate::ops::operation;
    use crate::Op;

    #[test]
    fn test_tree_traversal() {
        // Create a simple tree:
        //       1
        //      / \
        //     2   3
        //    /
        //   4
        let leaf = Op::value(4.0);
        let node2 = TreeNode::with_children(Op::value(2.0), vec![TreeNode::new(leaf)]);

        let node3 = TreeNode::new(Op::value(3.0));

        let root = Tree::new(TreeNode::with_children(Op::value(1.0), vec![node2, node3]));

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
