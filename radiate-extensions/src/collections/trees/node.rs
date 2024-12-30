use crate::ops::operation::Operation;
use radiate::engines::genome::genes::gene::{Gene, Valid};

use super::TreeIterator;
use crate::ops::operation::Arity;

#[derive(PartialEq)]
pub struct TreeNode<T> {
    pub value: Operation<T>,
    pub children: Option<Vec<TreeNode<T>>>,
}

impl<T> TreeNode<T> {
    pub fn new(val: Operation<T>) -> Self {
        TreeNode {
            value: val,
            children: None,
        }
    }

    pub fn with_children(val: Operation<T>, children: Vec<TreeNode<T>>) -> Self {
        TreeNode {
            value: val,
            children: Some(children),
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    pub fn add_child(&mut self, child: TreeNode<T>) {
        if let Some(children) = self.children.as_mut() {
            children.push(child);
        } else {
            self.children = Some(vec![child]);
        }
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

    pub fn swap_subtrees(&mut self, other: &mut TreeNode<T>, self_idx: usize, other_idx: usize) {
        if let (Some(self_subtree), Some(other_subtree)) =
            (self.get_mut(self_idx), other.get_mut(other_idx))
        {
            std::mem::swap(self_subtree, other_subtree);
        }
    }

    pub fn get(&self, index: usize) -> Option<&TreeNode<T>> {
        if index == 0 {
            return Some(self);
        }

        if let Some(children) = self.children.as_ref() {
            let mut count = 0;
            for child in children {
                let size = child.size();
                if index <= count + size {
                    return child.get(index - count - 1);
                }
                count += size;
            }
        }

        None
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

impl<T: Clone> Clone for TreeNode<T> {
    fn clone(&self) -> Self {
        TreeNode {
            value: self.value.clone(),
            children: self.children.as_ref().map(|children| children.to_vec()),
        }
    }
}

impl<T> Gene for TreeNode<T>
where
    T: Clone + PartialEq + Default,
{
    type Allele = Operation<T>;

    fn allele(&self) -> &Self::Allele {
        &self.value
    }

    fn new_instance(&self) -> Self {
        TreeNode {
            value: self.value.new_instance(),
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
            children: self.children.as_ref().map(|children| children.to_vec()),
        }
    }
}

impl<T> Valid for TreeNode<T> {
    fn is_valid(&self) -> bool {
        for node in self.iter_breadth_first() {
            match node.value.arity() {
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
