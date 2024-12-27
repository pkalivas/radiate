use crate::architects::schema::node_types::NodeType;
use uuid::Uuid;

use std::cell::RefCell;
use std::rc::Rc;

use super::{NodeBehavior, NodeCell, RefNodeCell};

pub struct TreeNode<T>
where
    T: Clone + PartialEq + Default,
{
    pub cell: RefNodeCell<T>,
    pub children: Vec<TreeNode<T>>,
}

impl<T> TreeNode<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(value: T) -> Self {
        Self {
            cell: Rc::new(RefCell::new(NodeCell::new(value))),
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: TreeNode<T>) {
        self.children.push(child);
    }

    pub fn children(&self) -> &[TreeNode<T>] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut [TreeNode<T>] {
        &mut self.children
    }

    pub fn traverse_pre_order<F>(&self, f: &mut F)
    where
        F: FnMut(&TreeNode<T>),
    {
        f(self);
        for child in &self.children {
            child.traverse_pre_order(f);
        }
    }

    pub fn traverse_post_order<F>(&self, f: &mut F)
    where
        F: FnMut(&TreeNode<T>),
    {
        for child in &self.children {
            child.traverse_post_order(f);
        }
        f(self);
    }
}

impl<T> NodeBehavior for TreeNode<T>
where
    T: Clone + PartialEq + Default,
{
    type Value = T;
    type Node = TreeNode<T>;

    fn node_type(&self) -> NodeType {
        if self.children.is_empty() {
            NodeType::Leaf
        } else {
            NodeType::Gate
        }
    }

    fn id(&self) -> Uuid {
        self.cell.borrow().id()
    }
}

impl<T> Clone for TreeNode<T>
where
    T: Clone + PartialEq + Default,
{
    fn clone(&self) -> Self {
        let mut new_tree = TreeNode::new(self.cell.borrow().value().clone());
        for child in &self.children {
            new_tree.add_child(child.clone());
        }
        new_tree
    }
}
