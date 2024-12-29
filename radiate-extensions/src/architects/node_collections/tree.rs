use crate::{TreeIterator, TreeNode};
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

impl<T> Debug for Tree<T>
where
    T: Debug,
{
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

    use crate::expr;

    #[test]
    fn test_tree() {
        let mut tree_one = Tree::new(TreeNode::with_children(
            expr::add(),
            vec![
                TreeNode::new(expr::value(1.0)),
                TreeNode::new(expr::value(2.0)),
            ],
        ));

        let mut tree_two = Tree::new(TreeNode::with_children(
            expr::mul(),
            vec![
                TreeNode::new(expr::value(3.0)),
                TreeNode::new(expr::value(4.0)),
            ],
        ));

        // Swap the first child of each tree
        tree_one.as_mut().swap_subtrees(tree_two.as_mut(), 1, 1);

        // Verify swap using breadth-first traversal
        let values_one: Vec<_> = tree_one
            .iter_breadth_first()
            .filter_map(|n| match &n.value {
                expr::Operation::Const(_, v) => Some(*v),
                _ => None,
            })
            .collect();

        assert_eq!(values_one, vec![3.0, 2.0]);
    }
}
