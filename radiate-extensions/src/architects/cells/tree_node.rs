use super::ValueCell;

#[derive(Clone, PartialEq)]
pub struct TreeNode<T> {
    pub inner: ValueCell<T>,
    pub children: Option<Vec<TreeNode<T>>>,
}

impl<T> TreeNode<T> {
    pub fn new(inner: ValueCell<T>) -> Self {
        TreeNode {
            inner,
            children: None,
        }
    }

    pub fn add_child(&mut self, child: TreeNode<T>) {
        if self.children.is_none() {
            self.children = Some(vec![]);
        }

        self.children.as_mut().unwrap().push(child);
    }

    pub fn children(&self) -> Option<&Vec<TreeNode<T>>> {
        self.children.as_ref()
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<TreeNode<T>>> {
        self.children.as_mut()
    }
}

impl<T> AsRef<ValueCell<T>> for TreeNode<T> {
    fn as_ref(&self) -> &ValueCell<T> {
        self.inner.as_ref()
    }
}

impl<T> AsMut<ValueCell<T>> for TreeNode<T> {
    fn as_mut(&mut self) -> &mut ValueCell<T> {
        self.inner.as_mut()
    }
}
