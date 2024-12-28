use crate::architects::schema::{direction::Direction, node_types::NodeType};
use crate::expr::Expr;
use crate::schema::collection_type::CollectionType;
use radiate::engines::genome::genes::gene::{Gene, Valid};
use std::collections::HashSet;
use uuid::Uuid;

use super::expr::Arity;
use super::TreeIterator;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    /// Terminal nodes provide values without inputs (constants, variables)
    Provider,
    /// Internal nodes transform values (operations)
    Internal,
    /// Output nodes consume values (network outputs)
    Output,
}

impl Role {
    pub fn from_expr<T>(expr: &Expr<T>) -> Self {
        match expr.arity() {
            Arity::Zero => Role::Provider,
            _ => Role::Internal, // Default to Internal, Output must be explicit
        }
    }

    pub fn allows_incoming(&self) -> bool {
        matches!(self, Role::Internal | Role::Output)
    }

    pub fn allows_outgoing(&self) -> bool {
        matches!(self, Role::Provider | Role::Internal)
    }
}

#[derive(Clone, PartialEq)]
pub struct NodeCell<T> {
    pub value: Expr<T>,
    pub id: Uuid,
    pub role: Role,
}

impl<T> NodeCell<T> {
    pub fn new(value: Expr<T>) -> Self {
        NodeCell {
            role: Role::from_expr(&value),
            id: Uuid::new_v4(),
            value,
        }
    }

    pub fn provider(value: Expr<T>) -> Self {
        if Role::from_expr(&value) != Role::Provider {
            panic!("NodeCell::provider() requires a provider role");
        }

        NodeCell {
            role: Role::Provider,
            id: Uuid::new_v4(),
            value,
        }
    }

    pub fn internal(value: Expr<T>) -> Self {
        if Role::from_expr(&value) != Role::Internal {
            panic!("NodeCell::internal() requires an internal role");
        }

        NodeCell {
            role: Role::Internal,
            id: Uuid::new_v4(),
            value,
        }
    }

    pub fn output(value: Expr<T>) -> Self {
        if value.arity() == Arity::Zero {
            panic!("NodeCell::output() requires a non-zero arity");
        }

        NodeCell {
            role: Role::Output,
            id: Uuid::new_v4(),
            value,
        }
    }
}

#[derive(PartialEq)]
pub struct TreeNode<T> {
    pub cell: NodeCell<T>,
    pub children: Option<Vec<TreeNode<T>>>,
}

impl<T> TreeNode<T> {
    pub fn new<C>(cell: C) -> Self
    where
        C: Into<NodeCell<T>>,
    {
        TreeNode {
            cell: cell.into(),
            children: None,
        }
    }

    pub fn with_children<C>(cell: C, children: Vec<TreeNode<T>>) -> Self
    where
        C: Into<NodeCell<T>>,
    {
        TreeNode {
            cell: cell.into(),
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

impl<T> AsRef<NodeCell<T>> for TreeNode<T> {
    fn as_ref(&self) -> &NodeCell<T> {
        &self.cell
    }
}

impl<T> AsMut<NodeCell<T>> for TreeNode<T> {
    fn as_mut(&mut self) -> &mut NodeCell<T> {
        &mut self.cell
    }
}

impl<T: Clone> Clone for TreeNode<T> {
    fn clone(&self) -> Self {
        TreeNode {
            cell: self.cell.clone(),
            children: self.children.as_ref().map(|children| children.to_vec()),
        }
    }
}

impl<T> Gene for TreeNode<T>
where
    T: Clone + PartialEq + Default,
{
    type Allele = Expr<T>;

    fn allele(&self) -> &Self::Allele {
        &self.cell.value
    }

    fn new_instance(&self) -> Self {
        TreeNode {
            cell: NodeCell {
                value: self.cell.value.new_instance(),
                ..self.cell.clone()
            },
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
            cell: NodeCell {
                value: allele.clone(),
                ..self.cell.clone()
            },
            children: self.children.as_ref().map(|children| children.to_vec()),
        }
    }
}

impl<T> Valid for TreeNode<T> {
    fn is_valid(&self) -> bool {
        for node in self.iter_breadth_first() {
            match node.cell.role {
                Role::Provider => {
                    if node.children.is_some() {
                        return false;
                    }
                }
                Role::Internal => {
                    if node.children.is_none() {
                        return false;
                    } else if node.cell.value.arity() == Arity::Zero {
                        return false;
                    } else if let Arity::Nary(n) = node.cell.value.arity() {
                        if node.children.as_ref().unwrap().len() != n as usize {
                            return false;
                        }
                    }
                }
                Role::Output => return false, // Outputs don't make sense in a tree
            }
        }

        true
    }
}

#[derive(Clone, PartialEq)]
pub struct GraphNode<T> {
    pub cell: NodeCell<T>,
    pub enabled: bool,
    pub direction: Direction,
    pub index: usize,
    pub incoming: HashSet<usize>,
    pub outgoing: HashSet<usize>,
}

impl<T> GraphNode<T> {
    pub fn new(index: usize, cell: NodeCell<T>) -> Self {
        GraphNode {
            cell,
            index,
            enabled: true,
            direction: Direction::Forward,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
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

impl<T> AsRef<NodeCell<T>> for GraphNode<T> {
    fn as_ref(&self) -> &NodeCell<T> {
        &self.cell
    }
}

impl<T> AsMut<NodeCell<T>> for GraphNode<T> {
    fn as_mut(&mut self) -> &mut NodeCell<T> {
        &mut self.cell
    }
}

impl<T> Gene for GraphNode<T>
where
    T: Clone + PartialEq + Default,
{
    type Allele = Expr<T>;

    fn allele(&self) -> &Self::Allele {
        &self.cell.value
    }

    fn new_instance(&self) -> Self {
        GraphNode {
            cell: NodeCell {
                value: self.cell.value.new_instance(),
                ..self.cell.clone()
            },
            index: self.index,
            enabled: self.enabled,
            direction: self.direction,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        GraphNode {
            cell: NodeCell {
                value: allele.clone(),
                ..self.cell.clone()
            },
            index: self.index,
            enabled: self.enabled,
            direction: self.direction,
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
        match self.cell.role {
            Role::Provider => {
                if let Arity::Zero = self.cell.value.arity() {
                    self.incoming.is_empty() && !self.outgoing.is_empty()
                } else {
                    false
                }
            }
            Role::Internal => {
                if self.incoming.is_empty() || self.outgoing.is_empty() {
                    return false;
                } else if let Arity::Zero = self.cell.value.arity() {
                    return false;
                } else if let Arity::Nary(n) = self.cell.value.arity() {
                    return self.incoming.len() == n as usize;
                }

                true
            }
            Role::Output => {
                if self.incoming.is_empty() {
                    return false;
                } else if let Arity::Zero = self.cell.value.arity() {
                    return false;
                } else if let Arity::Nary(n) = self.cell.value.arity() {
                    return self.incoming.len() == n as usize;
                }

                true
            }
        }
    }
}

pub struct Node<T> {
    pub id: Uuid,
    pub index: usize,
    pub value: Expr<T>,
    pub collection_type: Option<CollectionType>,
    pub enabled: bool,
    pub node_type: NodeType,
    pub direction: Direction,
    pub incoming: HashSet<usize>,
    pub outgoing: HashSet<usize>,
}

impl<T> Node<T> {
    pub fn new(index: usize, node_type: NodeType, value: Expr<T>) -> Self {
        Self {
            id: Uuid::new_v4(),
            index,
            value,
            enabled: true,
            direction: Direction::Forward,
            collection_type: None,
            node_type,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }

    pub fn node_type(&self) -> &NodeType {
        &self.node_type
    }

    pub fn value(&self) -> &Expr<T> {
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

impl<T> Gene for Node<T>
where
    T: Clone + PartialEq + Default,
{
    type Allele = Expr<T>;

    fn allele(&self) -> &Expr<T> {
        &self.value
    }

    fn new_instance(&self) -> Node<T> {
        Node {
            id: Uuid::new_v4(),
            index: self.index,
            enabled: self.enabled,
            value: self.value.new_instance(),
            direction: self.direction,
            collection_type: self.collection_type,
            node_type: self.node_type,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }

    fn with_allele(&self, allele: &Expr<T>) -> Node<T> {
        Node {
            id: Uuid::new_v4(),
            index: self.index,
            value: allele.clone(),
            enabled: self.enabled,
            collection_type: self.collection_type,
            direction: self.direction,
            node_type: self.node_type,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }
}

impl<T> Valid for Node<T>
where
    T: Clone + PartialEq,
{
    fn is_valid(&self) -> bool {
        if let Some(coll_type) = &self.collection_type {
            if coll_type == &CollectionType::Graph {
                return match self.node_type {
                    NodeType::Input => self.incoming.is_empty() && !self.outgoing.is_empty(),
                    NodeType::Output => !self.incoming.is_empty(),
                    NodeType::Gate => self.incoming.len() == *self.value.arity() as usize,
                    NodeType::Aggregate => !self.incoming.is_empty() && !self.outgoing.is_empty(),
                    NodeType::Weight => self.incoming.len() == 1 && self.outgoing.len() == 1,
                    NodeType::Link => self.incoming.len() == 1 && !self.outgoing.is_empty(),
                    NodeType::Leaf => self.incoming.is_empty() && !self.outgoing.is_empty(),
                };
            } else if coll_type == &CollectionType::Tree {
                return match self.node_type {
                    NodeType::Input => self.incoming.is_empty() && !self.outgoing.is_empty(),
                    NodeType::Output => !self.incoming.is_empty(),
                    NodeType::Gate => self.outgoing.len() == *self.value.arity() as usize,
                    NodeType::Aggregate => !self.incoming.is_empty() && !self.outgoing.is_empty(),
                    NodeType::Weight => self.incoming.len() == 1 && self.outgoing.len() == 1,
                    NodeType::Link => self.incoming.len() == 1 && !self.outgoing.is_empty(),
                    NodeType::Leaf => !self.incoming.is_empty() && self.outgoing.is_empty(),
                };
            }
        }

        false
    }
}

impl<T> Clone for Node<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Node {
            id: self.id,
            index: self.index,
            enabled: self.enabled,
            value: self.value.clone(),
            collection_type: self.collection_type,
            direction: self.direction,
            node_type: self.node_type,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }
}

impl<T> PartialEq for Node<T>
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

impl<T> Default for Node<T>
where
    T: Default,
{
    fn default() -> Self {
        Node {
            id: Uuid::new_v4(),
            index: 0,
            enabled: true,
            value: Expr::default(),
            direction: Direction::Forward,
            node_type: NodeType::Input,
            collection_type: None,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }
}

impl<T> std::fmt::Display for Node<T>
where
    T: Clone + PartialEq,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.index)
    }
}

impl<T> std::fmt::Debug for Node<T>
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
