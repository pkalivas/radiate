use radiate::engines::genome::genes::gene::{Gene, Valid};
use std::collections::HashSet;
use uuid::Uuid;

use crate::architects::schema::{direction::Direction, node_types::NodeType};
use crate::operations::op::Ops;
use crate::schema::collection_type::CollectionType;

pub struct Node<T>
where
    T: Clone + PartialEq,
{
    pub id: Uuid,
    pub index: usize,
    pub value: Ops<T>,
    pub collection_type: Option<CollectionType>,
    pub enabled: bool,
    pub node_type: NodeType,
    pub direction: Direction,
    pub incoming: HashSet<usize>,
    pub outgoing: HashSet<usize>,
}

impl<T> Node<T>
where
    T: Clone + PartialEq,
{
    pub fn new(index: usize, node_type: NodeType, value: Ops<T>) -> Self {
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

    pub fn value(&self) -> &Ops<T> {
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

impl<T> Gene<Node<T>, Ops<T>> for Node<T>
where
    T: Clone + PartialEq + Default,
{
    fn allele(&self) -> &Ops<T> {
        &self.value
    }

    fn new_instance(&self) -> Node<T> {
        Node {
            id: Uuid::new_v4(),
            index: self.index,
            enabled: self.enabled,
            value: self.value.new_instance(),
            direction: self.direction.clone(),
            collection_type: self.collection_type.clone(),
            node_type: self.node_type.clone(),
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }

    fn from_allele(&self, allele: &Ops<T>) -> Node<T> {
        Node {
            id: Uuid::new_v4(),
            index: self.index,
            value: allele.clone(),
            enabled: self.enabled,
            collection_type: self.collection_type.clone(),
            direction: self.direction.clone(),
            node_type: self.node_type.clone(),
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
                    NodeType::Output => self.incoming.len() > 0,
                    NodeType::Gate => self.incoming.len() == self.value.arity() as usize,
                    NodeType::Aggregate => !self.incoming.is_empty() && !self.outgoing.is_empty(),
                    NodeType::Weight => self.incoming.len() == 1 && self.outgoing.len() == 1,
                    NodeType::Link => self.incoming.len() == 1 && self.outgoing.len() > 0,
                    NodeType::Leaf => self.incoming.is_empty() && self.outgoing.len() > 0,
                };
            } else if coll_type == &CollectionType::Tree {
                return match self.node_type {
                    NodeType::Input => self.incoming.is_empty() && !self.outgoing.is_empty(),
                    NodeType::Output => self.incoming.len() > 0,
                    NodeType::Gate => self.outgoing.len() == self.value.arity() as usize,
                    NodeType::Aggregate => !self.incoming.is_empty() && !self.outgoing.is_empty(),
                    NodeType::Weight => self.incoming.len() == 1 && self.outgoing.len() == 1,
                    NodeType::Link => self.incoming.len() == 1 && self.outgoing.len() > 0,
                    NodeType::Leaf => self.incoming.len() > 0 && self.outgoing.is_empty(),
                };
            }
        }

        false
    }
}

impl<T> Clone for Node<T>
where
    T: Clone + PartialEq,
{
    fn clone(&self) -> Self {
        Node {
            id: self.id.clone(),
            index: self.index.clone(),
            enabled: self.enabled,
            value: self.value.clone(),
            collection_type: self.collection_type.clone(),
            direction: self.direction.clone(),
            node_type: self.node_type.clone(),
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }
}

impl<T> PartialEq for Node<T>
where
    T: Clone + PartialEq,
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
    T: Clone + PartialEq + Default,
{
    fn default() -> Self {
        Node {
            id: Uuid::new_v4(),
            index: 0,
            enabled: true,
            value: Ops::default(),
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
