use crate::NodeCell;

#[derive(Clone, PartialEq, Default)]
pub struct TreeNode<T> {
    pub id: uuid::Uuid,
    pub cell: NodeCell<T>,
    pub children: Option<Vec<TreeNode<T>>>,
}

impl<T> TreeNode<T> {
    pub fn new(cell: NodeCell<T>) -> Self {
        TreeNode {
            id: uuid::Uuid::new_v4(),
            cell,
            children: None,
        }
    }

    pub fn with_children(cell: NodeCell<T>, children: Vec<TreeNode<T>>) -> Self {
        TreeNode {
            id: uuid::Uuid::new_v4(),
            cell,
            children: Some(children),
        }
    }

    pub fn children(&self) -> Option<&[TreeNode<T>]> {
        self.children.as_ref().map(|children| children.as_slice())
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
