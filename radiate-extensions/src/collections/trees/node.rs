use crate::NodeCell;
use radiate::engines::genome::genes::gene::{Gene, Valid};

use super::TreeIterator;
use crate::ops::operation::Arity;

#[derive(PartialEq)]
pub struct TreeNode<C: NodeCell> {
    value: C,
    children: Option<Vec<TreeNode<C>>>,
}

impl<C: NodeCell> TreeNode<C> {
    pub fn new(val: C) -> Self {
        TreeNode {
            value: val,
            children: None,
        }
    }

    pub fn with_children(val: C, children: Vec<TreeNode<C>>) -> Self {
        TreeNode {
            value: val,
            children: Some(children),
        }
    }

    pub fn value(&self) -> &C {
        &self.value
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    pub fn add_child(&mut self, child: TreeNode<C>) {
        if let Some(children) = self.children.as_mut() {
            children.push(child);
        } else {
            self.children = Some(vec![child]);
        }
    }

    pub fn children(&self) -> Option<&Vec<TreeNode<C>>> {
        self.children.as_ref()
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<TreeNode<C>>> {
        self.children.as_mut()
    }

    pub fn size(&self) -> usize {
        if let Some(children) = self.children.as_ref() {
            children.iter().fold(1, |acc, child| acc + child.size())
        } else {
            1
        }
    }

    pub fn swap_subtrees(&mut self, other: &mut TreeNode<C>, self_idx: usize, other_idx: usize) {
        if let (Some(self_subtree), Some(other_subtree)) =
            (self.get_mut(self_idx), other.get_mut(other_idx))
        {
            std::mem::swap(self_subtree, other_subtree);
        }
    }

    pub fn get(&self, index: usize) -> Option<&TreeNode<C>> {
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

    pub fn get_mut(&mut self, index: usize) -> Option<&mut TreeNode<C>> {
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

impl<C: NodeCell + Clone> Clone for TreeNode<C> {
    fn clone(&self) -> Self {
        TreeNode {
            value: self.value.clone(),
            children: self.children.as_ref().map(|children| children.to_vec()),
        }
    }
}

impl<C> Gene for TreeNode<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    type Allele = C;

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
                    .collect::<Vec<TreeNode<C>>>()
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

impl<C: NodeCell> Valid for TreeNode<C> {
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
