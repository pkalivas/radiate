use super::NodeCell;

#[derive(PartialEq)]
pub struct TreeNode<T> {
    pub cell: NodeCell<T>,
    pub children: Option<Vec<TreeNode<T>>>,
}

impl<T> TreeNode<T> {
    pub fn new(cell: NodeCell<T>) -> Self {
        TreeNode {
            cell,
            children: None,
        }
    }

    pub fn with_children(cell: NodeCell<T>, children: Vec<TreeNode<T>>) -> Self {
        TreeNode {
            cell,
            children: Some(children),
        }
    }

    pub fn children(&self) -> Option<&Vec<TreeNode<T>>> {
        self.children.as_ref()
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<TreeNode<T>>> {
        self.children.as_mut()
    }
}

impl<T> AsRef<NodeCell<T>> for TreeNode<T> {
    fn as_ref(&self) -> &NodeCell<T> {
        &self.cell
    }
}

impl<T> AsMut<NodeCell<T>> for TreeNode<T> {
    fn as_mut(&mut self) -> &mut NodeCell<T> {
        &mut self.cell
    }
}

impl<T: Clone> Clone for TreeNode<T> {
    fn clone(&self) -> Self {
        TreeNode {
            cell: self.cell.clone(),
            children: self.children.as_ref().map(|children| {
                children
                    .iter()
                    .map(|child| child.clone())
                    .collect::<Vec<TreeNode<T>>>()
            }),
        }
    }
}
