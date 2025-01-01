use crate::{
    collections::{TreeIterator, TreeNode},
    NodeCell,
};
use std::fmt::Debug;

#[derive(Clone, PartialEq, Default)]
pub struct Tree<C: NodeCell> {
    root: Option<TreeNode<C>>,
}

impl<C: NodeCell> Tree<C> {
    pub fn new(root: TreeNode<C>) -> Self {
        Tree { root: Some(root) }
    }

    pub fn root(&self) -> Option<&TreeNode<C>> {
        self.root.as_ref()
    }

    pub fn root_mut(&mut self) -> Option<&mut TreeNode<C>> {
        self.root.as_mut()
    }
}

impl<C: NodeCell> AsRef<TreeNode<C>> for Tree<C> {
    fn as_ref(&self) -> &TreeNode<C> {
        self.root.as_ref().unwrap()
    }
}

impl<C: NodeCell> AsMut<TreeNode<C>> for Tree<C> {
    fn as_mut(&mut self) -> &mut TreeNode<C> {
        self.root.as_mut().unwrap()
    }
}

impl<C: NodeCell + Debug> Debug for Tree<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tree {{\n")?;
        for node in self.iter_breadth_first() {
            write!(f, "  {:?},\n", node.value)?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{ops::operation, Op};

    #[test]
    fn test_tree() {
        let mut tree_one = Tree::new(TreeNode::with_children(
            Op::add(),
            vec![
                TreeNode::new(Op::value(1.0)),
                TreeNode::new(Op::value(2.0)),
            ],
        ));

        let mut tree_two = Tree::new(TreeNode::with_children(
            Op::mul(),
            vec![
                TreeNode::new(Op::value(3.0)),
                TreeNode::new(Op::value(4.0)),
            ],
        ));

        // Swap the first child of each tree
        tree_one.as_mut().swap_subtrees(tree_two.as_mut(), 1, 1);

        // Verify swap using breadth-first traversal
        let values_one: Vec<_> = tree_one
            .iter_breadth_first()
            .filter_map(|n| match &n.value {
                operation::Op::Const(_, v) => Some(*v),
                _ => None,
            })
            .collect();

        assert_eq!(values_one, vec![3.0, 2.0]);
    }
}
