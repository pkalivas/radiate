use super::TreeIterator;
use crate::{node::Node, Arity, NodeType};
use radiate::engines::genome::gene::{Gene, Valid};

#[derive(PartialEq)]
pub struct TreeNode<T> {
    value: T,
    arity: Option<Arity>,
    children: Option<Vec<TreeNode<T>>>,
}

impl<T> TreeNode<T> {
    pub fn new(val: T) -> Self {
        TreeNode {
            value: val,
            arity: None,
            children: None,
        }
    }

    pub fn with_arity(val: T, arity: Arity) -> Self {
        TreeNode {
            value: val,
            arity: Some(arity),
            children: None,
        }
    }

    pub fn with_children<N>(val: T, children: Vec<N>) -> Self
    where
        N: Into<TreeNode<T>>,
    {
        TreeNode {
            value: val,
            arity: None,
            children: Some(children.into_iter().map(|n| n.into()).collect()),
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    pub fn add_child(&mut self, child: impl Into<TreeNode<T>>) {
        let node = child.into();
        if let Some(children) = self.children.as_mut() {
            children.push(node);
        } else {
            self.children = Some(vec![node]);
        }
    }

    pub fn attach(mut self, other: impl Into<TreeNode<T>>) -> Self {
        self.add_child(other);
        self
    }

    pub fn children(&self) -> Option<&Vec<TreeNode<T>>> {
        self.children.as_ref()
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<TreeNode<T>>> {
        self.children.as_mut()
    }

    pub fn size(&self) -> usize {
        if let Some(children) = self.children.as_ref() {
            children.iter().fold(1, |acc, child| acc + child.size())
        } else {
            1
        }
    }

    pub fn height(&self) -> usize {
        if let Some(children) = self.children.as_ref() {
            1 + children
                .iter()
                .map(|child| child.height())
                .max()
                .unwrap_or(0)
        } else {
            0
        }
    }

    pub fn swap_subtrees(&mut self, other: &mut TreeNode<T>, self_idx: usize, other_idx: usize) {
        if let (Some(self_subtree), Some(other_subtree)) =
            (self.get_mut(self_idx), other.get_mut(other_idx))
        {
            std::mem::swap(self_subtree, other_subtree);
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut TreeNode<T>> {
        if index == 0 {
            return Some(self);
        }

        if let Some(children) = self.children.as_mut() {
            let mut count = 0;
            for child in children {
                let size = child.size();
                if index <= count + size {
                    return child.get_mut(index - count - 1);
                }
                count += size;
            }
        }

        None
    }
}

impl<T> Node for TreeNode<T> {
    type Value = T;

    fn value(&self) -> &Self::Value {
        &self.value
    }

    fn node_type(&self) -> NodeType {
        if let Some(_) = self.children.as_ref() {
            NodeType::Vertex
        } else {
            NodeType::Leaf
        }
    }

    fn arity(&self) -> Arity {
        if let Some(arity) = self.arity {
            arity
        } else {
            if let Some(children) = self.children.as_ref() {
                Arity::Exact(children.len())
            } else {
                match self.node_type() {
                    NodeType::Leaf => Arity::Zero,
                    NodeType::Vertex => Arity::Any,
                    NodeType::Root => Arity::Any,
                    _ => Arity::Zero,
                }
            }
        }
    }
}

impl<T> Gene for TreeNode<T>
where
    T: Clone + PartialEq + Default,
{
    type Allele = T;

    fn allele(&self) -> &Self::Allele {
        &self.value
    }

    fn new_instance(&self) -> Self {
        TreeNode {
            value: self.value.clone(),
            arity: self.arity,
            children: self.children.as_ref().map(|children| {
                children
                    .iter()
                    .map(|child| child.new_instance())
                    .collect::<Vec<TreeNode<T>>>()
            }),
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        TreeNode {
            value: allele.clone(),
            arity: self.arity,
            children: self.children.as_ref().map(|children| children.to_vec()),
        }
    }
}

impl<T> Valid for TreeNode<T> {
    fn is_valid(&self) -> bool {
        for node in self.iter_breadth_first() {
            match node.arity() {
                Arity::Zero => {
                    if node.children.is_some() {
                        return false;
                    }
                }
                Arity::Exact(n) => {
                    if node.children.is_none()
                        || node.children.as_ref().unwrap().len() != n as usize
                    {
                        return false;
                    }
                }
                Arity::Any => {}
            }
        }

        true
    }
}

impl<T: Clone> Clone for TreeNode<T> {
    fn clone(&self) -> Self {
        TreeNode {
            value: self.value.clone(),
            arity: self.arity,
            children: self.children.as_ref().map(|children| children.to_vec()),
        }
    }
}
