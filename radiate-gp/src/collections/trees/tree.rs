use crate::collections::TreeIterator;
use crate::collections::TreeNode;
use crate::node::Node;

use std::fmt::Debug;

#[derive(Clone, PartialEq, Default)]
pub struct Tree<T> {
    root: Option<TreeNode<T>>,
}

impl<T> Tree<T> {
    pub fn new(root: TreeNode<T>) -> Self {
        Tree { root: Some(root) }
    }

    pub fn root(&self) -> Option<&TreeNode<T>> {
        self.root.as_ref()
    }

    pub fn root_mut(&mut self) -> Option<&mut TreeNode<T>> {
        self.root.as_mut()
    }

    pub fn take_root(self) -> Option<TreeNode<T>> {
        self.root
    }

    pub fn size(&self) -> usize {
        self.root.as_ref().map_or(0, |node| node.size())
    }

    pub fn height(&self) -> usize {
        self.root.as_ref().map_or(0, |node| node.height())
    }
}

impl<T> AsRef<TreeNode<T>> for Tree<T> {
    fn as_ref(&self) -> &TreeNode<T> {
        self.root.as_ref().unwrap()
    }
}

impl<T> AsMut<TreeNode<T>> for Tree<T> {
    fn as_mut(&mut self) -> &mut TreeNode<T> {
        self.root.as_mut().unwrap()
    }
}

impl<T: Debug> Debug for Tree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tree {{\n")?;
        for node in self.iter_breadth_first() {
            write!(f, "  {:?},\n", node.value())?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::Op;

    #[test]
    fn test_tree() {
        let mut tree_one = Tree::new(TreeNode::with_children(
            Op::add(),
            vec![TreeNode::new(Op::value(1.0)), TreeNode::new(Op::value(2.0))],
        ));

        let mut tree_two = Tree::new(TreeNode::with_children(
            Op::mul(),
            vec![TreeNode::new(Op::value(3.0)), TreeNode::new(Op::value(4.0))],
        ));

        // Swap the first child of each tree
        tree_one.as_mut().swap_subtrees(tree_two.as_mut(), 1, 1);

        // Verify swap using breadth-first traversal
        let values_one = tree_one
            .iter_breadth_first()
            .filter_map(|n| match &n.value() {
                Op::Const(_, v) => Some(*v),
                _ => None,
            })
            .collect::<Vec<f32>>();

        assert_eq!(values_one, vec![3.0, 2.0]);
    }
}
