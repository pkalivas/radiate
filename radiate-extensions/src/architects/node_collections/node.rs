use crate::operation::Operation;
use radiate::engines::genome::genes::gene::{Gene, Valid};
use std::collections::HashSet;
use uuid::Uuid;

use super::operation::Arity;
use super::TreeIterator;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Input,
    Output,
    Gate,
    Aggregate,
    Weight,
    Vertex,
    Edge,
}

pub struct GraphNode<T> {
    pub value: Operation<T>,
    pub id: Uuid,
    pub index: usize,
    pub enabled: bool,
    pub node_type: NodeType,
    pub direction: Direction,
    pub incoming: HashSet<usize>,
    pub outgoing: HashSet<usize>,
}

impl<T> GraphNode<T> {
    pub fn new(index: usize, node_type: NodeType, value: Operation<T>) -> Self {
        Self {
            id: Uuid::new_v4(),
            index,
            value,
            enabled: true,
            direction: Direction::Forward,
            node_type,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }

    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }

    pub fn value(&self) -> &Operation<T> {
        &self.value
    }

    pub fn is_recurrent(&self) -> bool {
        self.direction == Direction::Backward
            || self.incoming.contains(&self.index)
            || self.outgoing.contains(&self.index)
    }

    pub fn incoming(&self) -> &HashSet<usize> {
        &self.incoming
    }

    pub fn outgoing(&self) -> &HashSet<usize> {
        &self.outgoing
    }

    pub fn incoming_mut(&mut self) -> &mut HashSet<usize> {
        &mut self.incoming
    }

    pub fn outgoing_mut(&mut self) -> &mut HashSet<usize> {
        &mut self.outgoing
    }
}

impl<T> Gene for GraphNode<T>
where
    T: Clone + PartialEq + Default,
{
    type Allele = Operation<T>;

    fn allele(&self) -> &Operation<T> {
        &self.value
    }

    fn new_instance(&self) -> GraphNode<T> {
        GraphNode {
            id: Uuid::new_v4(),
            index: self.index,
            enabled: self.enabled,
            value: self.value.new_instance(),
            direction: self.direction,
            node_type: self.node_type,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }

    fn with_allele(&self, allele: &Operation<T>) -> GraphNode<T> {
        GraphNode {
            id: Uuid::new_v4(),
            index: self.index,
            value: allele.clone(),
            enabled: self.enabled,
            direction: self.direction,
            node_type: self.node_type,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }
}

impl<T> Valid for GraphNode<T>
where
    T: Clone + PartialEq,
{
    fn is_valid(&self) -> bool {
        match self.node_type {
            NodeType::Input => {
                self.incoming.is_empty()
                    && !self.outgoing.is_empty()
                    && self.value.arity() == Arity::Zero
            }
            NodeType::Output => !self.incoming.is_empty() && self.value.arity() == Arity::Any,
            NodeType::Gate => {
                self.incoming.len() == *self.value.arity() as usize && !self.outgoing.is_empty()
            }
            NodeType::Aggregate => {
                !self.incoming.is_empty()
                    && !self.outgoing.is_empty()
                    && self.value.arity() == Arity::Any
            }
            NodeType::Weight => {
                self.incoming.len() == 1
                    && self.outgoing.len() == 1
                    && self.value.arity() == Arity::Exact(1)
            }
            NodeType::Vertex => {
                if self.value.arity() == Arity::Any {
                    !self.incoming.is_empty() && !self.outgoing.is_empty()
                } else if let Arity::Exact(n) = self.value.arity() {
                    self.incoming.len() == n as usize && !self.outgoing.is_empty()
                } else {
                    self.incoming.is_empty() && !self.outgoing.is_empty()
                }
            }
            NodeType::Edge => {
                if self.value.arity() == Arity::Exact(1) {
                    return self.incoming.len() == 1 && self.outgoing.len() == 1;
                }

                false
            }
        }
    }
}

impl<T> Clone for GraphNode<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        GraphNode {
            id: self.id,
            index: self.index,
            enabled: self.enabled,
            value: self.value.clone(),
            direction: self.direction,
            node_type: self.node_type,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }
}

impl<T> PartialEq for GraphNode<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.index == other.index
            && self.value == other.value
            && self.direction == other.direction
            && self.node_type == other.node_type
            && self.incoming == other.incoming
            && self.outgoing == other.outgoing
    }
}

impl<T> Default for GraphNode<T>
where
    T: Default + Clone,
{
    fn default() -> Self {
        GraphNode {
            id: Uuid::new_v4(),
            index: 0,
            enabled: true,
            value: Operation::default(),
            direction: Direction::Forward,
            node_type: NodeType::Input,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }
}

impl<T> std::fmt::Display for GraphNode<T>
where
    T: Clone + PartialEq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

impl<T> std::fmt::Debug for GraphNode<T>
where
    T: Clone + PartialEq + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let incoming = self
            .incoming
            .iter()
            .map(|idx| idx.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        write!(
            f,
            "[{:<3}] {:>10?} :: {:<12} E: {:<5} V:{:<5} R:{:<5} {:<2} {:<2} < [{}]",
            self.index,
            format!("{:?}", self.node_type)[..3].to_owned(),
            format!("{:?}", self.value).to_owned(),
            self.enabled,
            self.is_valid(),
            self.is_recurrent(),
            self.incoming.len(),
            self.outgoing.len(),
            incoming
        )
    }
}
