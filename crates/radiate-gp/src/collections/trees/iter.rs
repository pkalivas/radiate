use crate::collections::{Tree, TreeNode};
use std::{collections::VecDeque, marker::PhantomData};

use super::TreeChromosome;

/// Tree traversal iterators for pre-order, post-order, and breadth-first search.
/// These iterators allow for efficient traversal of tree structures, providing
/// a way to visit each node in the tree in different orders.
///
/// # PreOrderIterator
/// The `PreOrderIterator` visits nodes in pre-order, meaning it visits the root
/// node first, then recursively visits each child node from left to right.
///
/// # PostOrderIterator
/// The `PostOrderIterator` visits nodes in post-order, meaning it recursively
/// visits each child node from left to right, and then visits the root node.
///
/// # TreeBreadthFirstIterator
/// The `TreeBreadthFirstIterator` visits nodes in breadth-first order, meaning
/// it visits all nodes at the current depth before moving on to the next depth.
///
/// # Usage
/// To use these iterators, you can call the `iter_pre_order`, `iter_post_order`,
/// or `iter_breadth_first` methods on a [TreeNode] or [Tree] instance. These
/// methods return an iterator that can be used to traverse the tree in the
/// desired order.
///
/// # Example
/// ```rust
/// use radiate_gp::*;
///
/// // create a simple tree
/// let tree = Tree::new(TreeNode::new(1)
///     .attach(TreeNode::new(2)
///         .attach(TreeNode::new(4))
///         .attach(TreeNode::new(5)))
///     .attach(TreeNode::new(3)
///         .attach(TreeNode::new(6))
///        .attach(TreeNode::new(7))));
///
/// // iterate over the tree in pre-order
/// // Output: 1, 2, 4, 5, 3, 6, 7
/// let pre_order = tree.iter_pre_order().map(|n| n.value()).collect::<Vec<&i32>>();
/// assert_eq!(pre_order, vec![&1, &2, &4, &5, &3, &6, &7]);
///
/// // iterate over the tree in post-order
/// // Output: 4, 5, 2, 6, 7, 3, 1
/// let post_order = tree.iter_post_order().map(|n| n.value()).collect::<Vec<&i32>>();
/// assert_eq!(post_order, vec![&4, &5, &2, &6, &7, &3, &1]);
///
/// // iterate over the tree in breadth-first order
/// // Output: 1, 2, 3, 4, 5, 6, 7
/// let breadth_first = tree.iter_breadth_first().map(|n| n.value()).collect::<Vec<&i32>>();
/// assert_eq!(breadth_first, vec![&1, &2, &3, &4, &5, &6, &7]);
/// ```
///
pub trait TreeIterator<T> {
    fn iter_pre_order(&self) -> PreOrderIterator<T>;
    fn iter_post_order(&self) -> PostOrderIterator<T>;
    fn iter_breadth_first(&self) -> TreeBreadthFirstIterator<T>;
    fn apply<F: Fn(&mut TreeNode<T>)>(&mut self, visit_fn: F);
}

/// Implement the [TreeIterator] trait for [TreeNode]
///
/// This allows for traversal of a single node and its children in pre-order, post-order, and breadth-first order.
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

    fn apply<F: Fn(&mut TreeNode<T>)>(&mut self, visit_fn: F) {
        let visitor = TreeVisitor::new(visit_fn);
        visitor.visit(self);
    }
}

/// Implement the [TreeIterator] trait for [Tree]
///
/// This allows for traversal of the entire tree in pre-order, post-order, and breadth-first order.
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

    fn apply<F: Fn(&mut TreeNode<T>)>(&mut self, visit_fn: F) {
        let visitor = TreeVisitor::new(visit_fn);
        if let Some(root) = self.root_mut() {
            visitor.visit(root);
        }
    }
}

impl<T> TreeIterator<T> for TreeChromosome<T> {
    fn iter_pre_order(&self) -> PreOrderIterator<T> {
        self.root().iter_pre_order()
    }

    fn iter_post_order(&self) -> PostOrderIterator<T> {
        self.root().iter_post_order()
    }

    fn iter_breadth_first(&self) -> TreeBreadthFirstIterator<T> {
        self.root().iter_breadth_first()
    }

    fn apply<F: Fn(&mut TreeNode<T>)>(&mut self, visit_fn: F) {
        self.root_mut().apply(visit_fn);
    }
}

pub struct PreOrderIterator<'a, T> {
    stack: Vec<&'a TreeNode<T>>,
}

impl<'a, T> Iterator for PreOrderIterator<'a, T> {
    type Item = &'a TreeNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().inspect(|node| {
            if let Some(children) = node.children() {
                for child in children.iter().rev() {
                    self.stack.push(child);
                }
            }
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

pub struct TreeVisitor<T, F>
where
    F: Fn(&mut TreeNode<T>),
{
    visitor: F,
    _marker: PhantomData<T>,
}

impl<T, F> TreeVisitor<T, F>
where
    F: Fn(&mut TreeNode<T>),
{
    pub fn new(visitor: F) -> Self {
        TreeVisitor {
            visitor,
            _marker: PhantomData,
        }
    }

    pub fn visit(&self, node: &mut TreeNode<T>) {
        (self.visitor)(node);

        if let Some(children) = node.children_mut() {
            for child in children.iter_mut() {
                self.visit(child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Op;
    use crate::collections::{Tree, TreeNode};
    use crate::node::Node;

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
                Op::Const(_, v) => *v,
                _ => panic!("Expected constant but got {:?}", n.value()),
            })
            .collect();
        assert_eq!(pre_order, vec![1.0, 2.0, 4.0, 3.0]);

        // Test post-order
        let post_order: Vec<f32> = root
            .iter_post_order()
            .map(|n| match &n.value() {
                Op::Const(_, v) => *v,
                _ => panic!("Expected constant"),
            })
            .collect();
        assert_eq!(post_order, vec![4.0, 2.0, 3.0, 1.0]);

        // Test breadth-first
        let bfs: Vec<f32> = root
            .iter_breadth_first()
            .map(|n| match &n.value() {
                Op::Const(_, v) => *v,
                _ => panic!("Expected constant"),
            })
            .collect();
        assert_eq!(bfs, vec![1.0, 2.0, 3.0, 4.0]);
    }
}
