use crate::ops::{Arity, Op};
use crate::{Node, NodeCell};
use radiate::{Gene, Valid};
use std::collections::HashSet;
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Input,
    Output,
    Vertex,
    Edge,
}

#[derive(Clone, PartialEq)]
pub struct GraphNode<C: NodeCell> {
    pub value: C,
    pub id: Uuid,
    pub index: usize,
    pub enabled: bool,
    pub node_type: NodeType,
    pub direction: Direction,
    pub incoming: HashSet<usize>,
    pub outgoing: HashSet<usize>,
}

impl<C: NodeCell> GraphNode<C> {
    pub fn new(index: usize, node_type: NodeType, value: C) -> Self {
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
        if self.value.arity() == Arity::Any {
            return false;
        }

        self.incoming.len() == *self.value.arity()
    }
}

impl<T> Node<Op<T>> for GraphNode<Op<T>>
where
    T: Clone + PartialEq,
{
    fn arity(&self) -> Arity {
        self.value.arity()
    }

    fn cell(&self, _index: usize) -> &Op<T> {
        &self.value
    }

    fn cell_mut(&mut self, _index: usize) -> &mut Op<T> {
        &mut self.value
    }
}

impl<C: NodeCell> Gene for GraphNode<C>
where
    C: Clone + PartialEq + Default,
{
    type Allele = C;

    fn allele(&self) -> &C {
        &self.value
    }

    fn new_instance(&self) -> GraphNode<C> {
        GraphNode {
            id: Uuid::new_v4(),
            index: self.index,
            enabled: self.enabled,
            value: self.value.clone(),
            direction: self.direction,
            node_type: self.node_type,
            incoming: self.incoming.clone(),
            outgoing: self.outgoing.clone(),
        }
    }

    fn with_allele(&self, allele: &C) -> GraphNode<C> {
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

impl<C: NodeCell + Clone + PartialEq> Valid for GraphNode<C> {
    fn is_valid(&self) -> bool {
        match self.node_type {
            NodeType::Input => {
                self.incoming.is_empty()
                    && !self.outgoing.is_empty()
                    && self.value.arity() == Arity::Zero
            }
            NodeType::Output => !self.incoming.is_empty() && self.value.arity() == Arity::Any,
            NodeType::Vertex => {
                if self.value.arity() == Arity::Any {
                    !self.incoming.is_empty() && !self.outgoing.is_empty()
                } else if let Arity::Exact(n) = self.value.arity() {
                    self.incoming.len() == n && !self.outgoing.is_empty()
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

impl<C: NodeCell + Default> Default for GraphNode<C> {
    fn default() -> Self {
        GraphNode {
            id: Uuid::new_v4(),
            index: 0,
            enabled: true,
            value: C::default(),
            direction: Direction::Forward,
            node_type: NodeType::Input,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }
}

impl<C: NodeCell + Debug + PartialEq + Clone> Debug for GraphNode<C> {
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
