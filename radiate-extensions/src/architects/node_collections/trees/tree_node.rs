use std::{cell::RefCell, rc::Rc};

use uuid::Uuid;

use crate::{schema::collection_type::CollectionType, NodeBehavior, NodeType};

pub struct TreeNode<T>
where
    T: Clone + PartialEq + Default,
{
    pub id: Uuid,
    pub value: T,
    pub children: Vec<Rc<RefCell<TreeNode<T>>>>,
}

impl<T> TreeNode<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(value: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            value,
            children: Vec::new(),
        }
    }

    pub fn add_child(&mut self, child: TreeNode<T>) {
        self.children.push(Rc::new(RefCell::new(child)));
    }

    pub fn children(&self) -> &[Rc<RefCell<TreeNode<T>>>] {
        &self.children
    }

    pub fn children_mut(&mut self) -> &mut Vec<Rc<RefCell<TreeNode<T>>>> {
        &mut self.children
    }

    pub fn traverse_pre_order<F>(&self, f: &mut F)
    where
        F: FnMut(&TreeNode<T>),
    {
        f(self);
        for child in &self.children {
            child.borrow().traverse_pre_order(f);
        }
    }

    pub fn traverse_post_order<F>(&self, f: &mut F)
    where
        F: FnMut(&TreeNode<T>),
    {
        for child in &self.children {
            child.borrow().traverse_post_order(f);
        }
        f(self);
    }
}

impl<T> NodeBehavior<T> for TreeNode<T>
where
    T: Clone + PartialEq + Default,
{
    fn value(&self) -> &T {
        &self.value
    }

    fn node_type(&self) -> &NodeType {
        if self.children.is_empty() {
            &NodeType::Leaf
        } else {
            &NodeType::Gate
        }
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn id(&self) -> Uuid {
        self.id
    }

    fn collection_type(&self) -> CollectionType {
        CollectionType::Tree
    }
}

impl<T> Clone for TreeNode<T>
where
    T: Clone + PartialEq + Default,
{
    fn clone(&self) -> Self {
        let mut new_tree = TreeNode::new(self.value.clone());
        for child in &self.children {
            new_tree.add_child(child.borrow().clone());
        }
        new_tree
    }
}
