use super::TreeIterator;
use crate::{Arity, NodeType, node::Node};
use radiate::engines::genome::gene::{Gene, Valid};
use std::fmt::Debug;

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

    pub fn detach(&mut self, index: usize) -> Option<TreeNode<T>> {
        if let Some(children) = self.children.as_mut() {
            if index < children.len() {
                return Some(children.remove(index));
            }
        }
        None
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
        let self_subtree = self.get_mut(self_idx);
        let other_subtree = other.get_mut(other_idx);

        if let (Some(self_sub), Some(other_sub)) = (self_subtree, other_subtree) {
            std::mem::swap(self_sub, other_sub);
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

    fn value_mut(&mut self) -> &mut Self::Value {
        &mut self.value
    }

    fn node_type(&self) -> NodeType {
        if self.children.as_ref().is_some() {
            NodeType::Vertex
        } else {
            NodeType::Leaf
        }
    }

    fn arity(&self) -> Arity {
        if let Some(arity) = self.arity {
            arity
        } else if let Some(children) = self.children.as_ref() {
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
                    if node.children.is_none() || node.children.as_ref().unwrap().len() != n {
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

impl<T: Debug> Debug for TreeNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fn pretty_print_lines<T: Debug>(
            node: &TreeNode<T>,
            prefix: &str,
            is_last: bool,
            result: &mut String,
        ) {
            let connector = if is_last { "└── " } else { "├── " };
            let new_str_to_print = format!("{}{}{:?}\n", prefix, connector, node.value());
            result.push_str(&new_str_to_print);

            if let Some(children) = &node.children {
                let len = children.len();
                for (i, child) in children.iter().enumerate() {
                    let is_last_child = i == len - 1;
                    let new_prefix = if is_last {
                        format!("{}    ", prefix)
                    } else {
                        format!("{}│   ", prefix)
                    };
                    pretty_print_lines(child, &new_prefix, is_last_child, result);
                }
            }
        }
        let mut result = String::new();
        pretty_print_lines(self, "", true, &mut result);
        write!(f, "{}", result)
    }
}

impl From<i32> for TreeNode<i32> {
    fn from(n: i32) -> Self {
        TreeNode::new(n)
    }
}

impl From<f64> for TreeNode<f64> {
    fn from(value: f64) -> Self {
        TreeNode::new(value)
    }
}

impl From<String> for TreeNode<String> {
    fn from(value: String) -> Self {
        TreeNode::new(value)
    }
}

impl From<bool> for TreeNode<bool> {
    fn from(value: bool) -> Self {
        TreeNode::new(value)
    }
}

impl From<char> for TreeNode<char> {
    fn from(value: char) -> Self {
        TreeNode::new(value)
    }
}

impl From<usize> for TreeNode<usize> {
    fn from(value: usize) -> Self {
        TreeNode::new(value)
    }
}
