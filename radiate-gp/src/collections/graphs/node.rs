use crate::node::Node;
use crate::{Arity, NodeType};
use radiate::{Gene, Valid};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use uuid::Uuid;

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
    direction: Direction,
    arity: Option<Arity>,
    incoming: HashSet<usize>,
    outgoing: HashSet<usize>,
}

impl<T> GraphNode<T> {
    pub fn new(index: usize, value: T) -> Self {
        GraphNode {
            id: Uuid::new_v4(),
            index,
            value,
            enabled: true,
            direction: Direction::Forward,
            arity: None,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }

    pub fn with_arity(index: usize, value: T, arity: Arity) -> Self {
        GraphNode {
            id: Uuid::new_v4(),
            index,
            value,
            enabled: true,
            direction: Direction::Forward,

            arity: Some(arity),
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
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

    pub fn is_locked(&self) -> bool {
        if self.arity() == Arity::Any {
            return false;
        }

        self.incoming.len() == *self.arity()
    }
}

impl<T> Node for GraphNode<T> {
    type Value = T;

    fn value(&self) -> &Self::Value {
        &self.value
    }

    fn node_type(&self) -> NodeType {
        // self.node_type

        let arity = self.arity();

        if let Arity::Any = arity {
            if self.outgoing.is_empty() {
                return NodeType::Output;
            } else {
                return NodeType::Vertex;
            }
        } else if let Arity::Exact(1) = arity {
            if self.incoming.len() == 1 && self.outgoing.len() == 1 {
                return NodeType::Edge;
            } else {
                return NodeType::Vertex;
            }
        } else if let Arity::Zero = arity {
            return NodeType::Input;
        } else {
            return NodeType::Vertex;
        }
    }

    fn arity(&self) -> Arity {
        self.arity.unwrap_or(Arity::Any)
        // self.arity.unwrap_or(match self.node_type {
        //     NodeType::Input => Arity::Zero,
        //     NodeType::Output => Arity::Any,
        //     NodeType::Vertex => Arity::Any,
        //     NodeType::Edge => Arity::Exact(1),
        //     NodeType::Leaf => Arity::Zero,
        //     NodeType::Root => Arity::Any,
        // })
    }
}

impl<T> Gene for GraphNode<T>
where
    T: Clone + PartialEq + Default,
{
    type Allele = T;

    fn allele(&self) -> &Self::Allele {
        self.value()
    }

    fn new_instance(&self) -> GraphNode<T> {
        GraphNode {
            id: Uuid::new_v4(),
            index: self.index,
            enabled: self.enabled,
            value: self.value.clone(),
            direction: self.direction,

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

            arity: self.arity,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }
}

impl<T> Valid for GraphNode<T> {
    fn is_valid(&self) -> bool {
        match self.node_type() {
            NodeType::Input => self.incoming.is_empty() && !self.outgoing.is_empty(),
            NodeType::Output => {
                (!self.incoming.is_empty())
                    && (self.incoming.len() == *self.arity() || self.arity() == Arity::Any)
            }
            NodeType::Vertex => {
                if !self.incoming.is_empty() && !self.outgoing.is_empty() {
                    if let Arity::Exact(n) = self.arity() {
                        return self.incoming.len() == n;
                    } else if self.arity() == Arity::Any {
                        return true;
                    }
                }
                false
            }
            NodeType::Edge => {
                if self.arity() == Arity::Exact(1) {
                    return self.incoming.len() == 1 && self.outgoing.len() == 1;
                }

                false
            }
            _ => false,
        }
    }
}

impl<T: Default> Default for GraphNode<T> {
    fn default() -> Self {
        GraphNode {
            id: Uuid::new_v4(),
            index: 0,
            enabled: true,
            value: Default::default(),
            direction: Direction::Forward,

            arity: None,
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
            self.arity()
        )
    }
}
