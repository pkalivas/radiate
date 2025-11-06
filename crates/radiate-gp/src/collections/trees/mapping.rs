use crate::{Node, Tree, TreeNode};

pub trait TreeMapper<T, U> {
    type Output;

    fn map<F>(&self, mapper_fn: F) -> Self::Output
    where
        F: Fn(&T) -> U;
}

impl<T, U> TreeMapper<T, U> for TreeNode<T> {
    type Output = TreeNode<U>;

    fn map<F>(&self, mapper_fn: F) -> Self::Output
    where
        F: Fn(&T) -> U,
    {
        fn map_tree_node<T, U, F>(node: &TreeNode<T>, mapper_fn: &F) -> TreeNode<U>
        where
            F: Fn(&T) -> U,
        {
            let mapped_node = mapper_fn(node.value());

            if let Some(children) = node.children() {
                TreeNode::from((
                    mapped_node,
                    children
                        .iter()
                        .map(|child| map_tree_node(child, mapper_fn))
                        .collect::<Vec<TreeNode<U>>>(),
                ))
            } else {
                TreeNode::new(mapped_node)
            }
        }

        map_tree_node(self, &mapper_fn)
    }
}

impl<T, U> TreeMapper<T, U> for Tree<T>
where
    U: Default,
{
    type Output = Tree<U>;

    fn map<F>(&self, mapper_fn: F) -> Self::Output
    where
        F: Fn(&T) -> U,
    {
        if let Some(root) = self.root() {
            Tree::new(root.map(mapper_fn))
        } else {
            Tree::default()
        }
    }
}
