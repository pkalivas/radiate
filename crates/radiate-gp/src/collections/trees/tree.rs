use crate::{TreeIterator, collections::TreeNode};
use std::fmt::Debug;

#[derive(Clone, PartialEq, Default)]
pub struct Tree<T> {
    root: Option<TreeNode<T>>,
}

impl<T> Tree<T> {
    pub fn new(root: impl Into<TreeNode<T>>) -> Self {
        Tree {
            root: Some(root.into()),
        }
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
            write!(f, "  {:?}\n", node)?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Node, NodeType, Op, TreeIterator};

    #[test]
    fn test_swap_subtrees() {
        let mut tree_one = Tree::new(
            TreeNode::new(Op::add())
                .attach(TreeNode::new(Op::constant(1.0)))
                .attach(TreeNode::new(Op::constant(2.0))),
        );

        let mut tree_two = Tree::new(
            TreeNode::new(Op::mul())
                .attach(TreeNode::new(Op::constant(3.0)))
                .attach(TreeNode::new(Op::constant(4.0))),
        );

        tree_one.as_mut().swap_subtrees(tree_two.as_mut(), 1, 1);

        let values_one = tree_one
            .iter_breadth_first()
            .filter_map(|n| match &n.value() {
                Op::Const(_, v) => Some(*v),
                _ => None,
            })
            .collect::<Vec<f32>>();

        assert_eq!(values_one, vec![3.0, 2.0]);
    }

    #[test]
    fn test_size() {
        let tree = Tree::new(
            TreeNode::new(Op::add())
                .attach(TreeNode::from(Op::constant(1.0)))
                .attach(TreeNode::from(Op::constant(2.0))),
        );

        assert_eq!(tree.size(), 3);
    }

    #[test]
    fn test_depth() {
        let store = vec![
            (NodeType::Vertex, vec![Op::add(), Op::sub(), Op::mul()]),
            (NodeType::Leaf, vec![Op::constant(1.0), Op::constant(2.0)]),
        ];

        let tree = Tree::with_depth(5, store);
        assert_eq!(tree.height(), 5);
    }
}
