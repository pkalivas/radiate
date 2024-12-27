use crate::{Expr, NodeBehavior, NodeCell, NodeType};
use uuid::Uuid;

pub struct TreeNode<T> {
    pub cell: NodeCell<T>,
    pub children: Vec<TreeNode<T>>,
}

impl<T> TreeNode<T> {
    pub fn new(value: Expr<T>) -> Self {
        Self {
            cell: NodeCell::new(value),
            children: Vec::new(),
        }
    }

    pub fn with_schema(value: NodeCell<T>) -> Self {
        Self {
            cell: value,
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
    type Value = Expr<T>;
    type Node = TreeNode<T>;

    fn node_type(&self) -> NodeType {
        if self.children.is_empty() {
            NodeType::Leaf
        } else {
            NodeType::Gate
        }
    }

    fn id(&self) -> Uuid {
        self.cell.id
    }

    fn value(&self) -> &Self::Value {
        &self.cell.value
    }
}

impl<T> Clone for TreeNode<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let mut new_tree = TreeNode::with_schema(self.cell.clone());
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
        self.cell.id == other.cell.id && self.cell == other.cell && self.children == other.children
    }
}
