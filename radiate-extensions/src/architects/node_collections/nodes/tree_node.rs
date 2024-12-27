use super::NodeBehavior;
use crate::architects::schema::node_types::NodeType;
use crate::{NodeCell, NodeSchema};
use uuid::Uuid;

pub struct TreeNode<T> {
    id: Uuid,
    value: NodeCell<T>,
    children: Vec<TreeNode<T>>,
}

impl<T> TreeNode<T> {
    pub fn new(value: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            value: NodeCell::new(value),
            children: Vec::new(),
        }
    }

    pub fn with_schema(value: NodeCell<T>) -> Self {
        Self {
            id: Uuid::new_v4(),
            value,
            children: Vec::new(),
        }
    }

    pub fn cell(&self) -> &NodeCell<T> {
        &self.value
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
        self.id
    }

    fn value(&self) -> &Self::Value {
        self.value.value()
    }
}

impl<T> Clone for TreeNode<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut new_tree = TreeNode::with_schema(self.value.clone());
        for child in &self.children {
            new_tree.add_child(child.clone());
        }
        new_tree
    }
}


impl<T> PartialEq for TreeNode<T>
where
    T: Clone + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.value == other.value
            && self.children == other.children
    }
}