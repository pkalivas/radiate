use crate::ops::Arity;
use crate::Op;
use radiate::{Gene, Valid};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Input,
    Output,
    Vertex,
    Edge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Clone, PartialEq)]
pub struct GraphNode<T> {
    value: T,
    id: Uuid,
    index: usize,
    enabled: bool,
    node_type: NodeType,
    direction: Direction,
    arity: Option<Arity>,
    incoming: HashSet<usize>,
    outgoing: HashSet<usize>,
}

impl<T> GraphNode<T> {
    pub fn new(index: usize, node_type: NodeType, value: impl Into<T>) -> Self {
        (index, node_type, value.into()).into()
    }

    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn value(&self) -> &T {
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

    pub fn set_arity(&mut self, arity: Arity) {
        self.arity = Some(arity);
    }

    pub fn arity(&self) -> Option<&Arity> {
        self.arity.as_ref()
    }

    pub fn is_locked(&self) -> bool {
        if let Some(arity) = self.arity {
            if let Arity::Exact(n) = arity {
                return self.incoming.len() == n;
            }

            return false;
        }

        false
        // if self.arity() == Arity::Any {
        //     return false;
        // }

        // self.incoming.len() == *self.arity()
    }
}

impl<T> Gene for GraphNode<T>
where
    T: Clone + PartialEq + Default,
{
    type Allele = T;

    fn allele(&self) -> &Self::Allele {
        &self.value
    }

    fn new_instance(&self) -> GraphNode<T> {
        GraphNode {
            id: Uuid::new_v4(),
            index: self.index,
            enabled: self.enabled,
            value: self.value.clone(),
            direction: self.direction,
            node_type: self.node_type,
            arity: self.arity,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> GraphNode<T> {
        GraphNode {
            id: Uuid::new_v4(),
            index: self.index,
            value: allele.clone(),
            enabled: self.enabled,
            direction: self.direction,
            node_type: self.node_type,
            arity: self.arity,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }
}

impl<T> Valid for GraphNode<T> {
    fn is_valid(&self) -> bool {
        if let Some(arity) = self.arity {
            return match self.node_type {
                NodeType::Input => self.incoming.is_empty() && !self.outgoing.is_empty(),
                NodeType::Output => {
                    (!self.incoming.is_empty())
                        && (self.incoming.len() == *arity || arity == Arity::Any)
                }
                NodeType::Vertex => {
                    if !self.incoming.is_empty() && !self.outgoing.is_empty() {
                        if let Arity::Exact(n) = arity {
                            return self.incoming.len() == n;
                        } else if arity == Arity::Any {
                            return true;
                        }
                    }
                    return false;
                }
                NodeType::Edge => {
                    if arity == Arity::Exact(1) {
                        return self.incoming.len() == 1 && self.outgoing.len() == 1;
                    }

                    false
                }
            };
        }

        true
    }
}

impl<T> From<(usize, NodeType, T)> for GraphNode<Op<T>>
where
    T: Into<Op<T>>,
{
    fn from((index, node_type, value): (usize, NodeType, T)) -> Self {
        let value = value.into(); // Convert value into Op<T>
        let arity = value.arity(); // Get arity from Op<T>

        println!("Creating GraphNode with arity: {:?}", arity);

        GraphNode {
            id: Uuid::new_v4(),
            index,
            value,
            enabled: true,
            arity: Some(arity), // Set the GraphNode's arity
            direction: Direction::Forward,
            node_type,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }
}
impl<T> From<(usize, NodeType, T)> for GraphNode<T> {
    fn from((index, node_type, value): (usize, NodeType, T)) -> Self {
        println!("HERERE");
        GraphNode {
            id: Uuid::new_v4(),
            index,
            value,
            enabled: true,
            arity: None, // Non-Op<T> values don't have an arity
            direction: Direction::Forward,
            node_type,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }
}

impl<T: Default> Default for GraphNode<T> {
    fn default() -> Self {
        GraphNode {
            id: Uuid::new_v4(),
            index: 0,
            enabled: true,
            value: T::default(),
            arity: None,
            direction: Direction::Forward,
            node_type: NodeType::Input,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }
}

impl<T: Debug + PartialEq + Clone> Debug for GraphNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let incoming = self
            .incoming
            .iter()
            .map(|idx| idx.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        write!(
            f,
            "[{:<3}] {:>10?} :: {:<12} E: {:<5} V:{:<5} R:{:<5} {:<2} {:<2} < [{}] > {:?}",
            self.index,
            format!("{:?}", self.node_type())[..3].to_owned(),
            format!("{:?}", self.value).to_owned(),
            self.enabled,
            self.is_valid(),
            self.is_recurrent(),
            self.incoming.len(),
            self.outgoing.len(),
            incoming,
            self.arity
        )
    }
}
