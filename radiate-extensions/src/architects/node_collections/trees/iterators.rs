use std::{cell::RefCell, rc::Rc};

use super::TreeNode;

pub struct TreeInOrderIterator<T>
where
    T: Clone + PartialEq + Default,
{
    stack: Vec<Rc<RefCell<TreeNode<T>>>>,
}

impl<T> TreeInOrderIterator<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(root: Rc<RefCell<TreeNode<T>>>) -> Self {
        let mut stack = Vec::new();
        stack.push(root.clone());
        TreeInOrderIterator { stack }
    }
}

impl<T> Iterator for TreeInOrderIterator<T>
where
    T: Clone + PartialEq + Default,
{
    type Item = Rc<RefCell<TreeNode<T>>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            for child in node.borrow().children() {
                self.stack.push(child.clone());
            }

            return Some(node);
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }

    fn count(self) -> usize {
        0
    }
}
