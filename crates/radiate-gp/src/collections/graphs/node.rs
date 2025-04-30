use crate::node::Node;
use crate::{Arity, NodeType};
use radiate_core::{Gene, Valid};
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct GraphNodeId(u64);

impl GraphNodeId {
    pub fn new() -> Self {
        static GRAPH_NODE_ID: AtomicU64 = AtomicU64::new(0);
        GraphNodeId(GRAPH_NODE_ID.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Clone, PartialEq)]
pub struct GraphNode<T> {
    value: T,
    id: GraphNodeId,
    index: usize,
    direction: Direction,
    node_type: Option<NodeType>,
    arity: Option<Arity>,
    incoming: BTreeSet<usize>,
    outgoing: BTreeSet<usize>,
}

impl<T> GraphNode<T> {
    pub fn new(index: usize, node_type: NodeType, value: T) -> Self {
        GraphNode {
            id: GraphNodeId::new(),
            index,
            value,
            direction: Direction::Forward,
            node_type: Some(node_type),
            arity: None,
            incoming: BTreeSet::new(),
            outgoing: BTreeSet::new(),
        }
    }

    pub fn with_arity(index: usize, node_type: NodeType, value: T, arity: Arity) -> Self {
        GraphNode {
            id: GraphNodeId::new(),
            index,
            value,
            direction: Direction::Forward,
            node_type: Some(node_type),
            arity: Some(arity),
            incoming: BTreeSet::new(),
            outgoing: BTreeSet::new(),
        }
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn id(&self) -> &GraphNodeId {
        &self.id
    }

    pub fn is_recurrent(&self) -> bool {
        self.direction == Direction::Backward
            || self.incoming.contains(&self.index)
            || self.outgoing.contains(&self.index)
    }

    pub fn incoming(&self) -> &BTreeSet<usize> {
        &self.incoming
    }

    pub fn outgoing(&self) -> &BTreeSet<usize> {
        &self.outgoing
    }

    pub fn incoming_mut(&mut self) -> &mut BTreeSet<usize> {
        &mut self.incoming
    }

    pub fn outgoing_mut(&mut self) -> &mut BTreeSet<usize> {
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

    fn value_mut(&mut self) -> &mut Self::Value {
        &mut self.value
    }

    fn node_type(&self) -> NodeType {
        if let Some(node_type) = self.node_type {
            return node_type;
        }

        let arity = self.arity();

        if let Arity::Any = arity {
            if self.outgoing.is_empty() && self.incoming.is_empty() {
                NodeType::Vertex
            } else if self.outgoing.is_empty() {
                NodeType::Output
            } else {
                NodeType::Vertex
            }
        } else if let Arity::Exact(1) = arity {
            if self.incoming.len() == 1 && self.outgoing.len() == 1 {
                NodeType::Edge
            } else {
                NodeType::Vertex
            }
        } else if let Arity::Zero = arity {
            NodeType::Input
        } else {
            NodeType::Vertex
        }
    }

    fn arity(&self) -> Arity {
        if let Some(node_type) = self.node_type {
            return self.arity.unwrap_or(match node_type {
                NodeType::Input => Arity::Zero,
                NodeType::Output => Arity::Any,
                NodeType::Vertex => Arity::Any,
                NodeType::Edge => Arity::Exact(1),
                NodeType::Leaf => Arity::Zero,
                NodeType::Root => Arity::Any,
            });
        }

        self.arity.unwrap_or(Arity::Any)
    }
}

impl<T> Gene for GraphNode<T>
where
    T: Clone + PartialEq,
{
    type Allele = T;

    fn allele(&self) -> &Self::Allele {
        self.value()
    }

    fn new_instance(&self) -> GraphNode<T> {
        GraphNode {
            id: GraphNodeId::new(),
            index: self.index,
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
            id: GraphNodeId::new(),
            index: self.index,
            value: allele.clone(),
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

impl<T> From<(usize, NodeType, T)> for GraphNode<T> {
    fn from((index, node_type, value): (usize, NodeType, T)) -> Self {
        GraphNode::new(index, node_type, value)
    }
}

impl<T: Default> From<(usize, T)> for GraphNode<T> {
    fn from((index, value): (usize, T)) -> Self {
        GraphNode {
            index,
            id: GraphNodeId::new(),
            value,
            direction: Direction::Forward,
            node_type: None,
            arity: None,
            incoming: BTreeSet::new(),
            outgoing: BTreeSet::new(),
        }
    }
}

impl<T> From<(usize, NodeType, T, Arity)> for GraphNode<T> {
    fn from((index, node_type, value, arity): (usize, NodeType, T, Arity)) -> Self {
        GraphNode::with_arity(index, node_type, value, arity)
    }
}

impl<T: Default> From<(usize, T, Arity)> for GraphNode<T> {
    fn from((index, value, arity): (usize, T, Arity)) -> Self {
        GraphNode {
            index,
            id: GraphNodeId::new(),
            value,
            direction: Direction::Forward,
            node_type: None,
            arity: Some(arity),
            incoming: BTreeSet::new(),
            outgoing: BTreeSet::new(),
        }
    }
}

impl<T: Default> Default for GraphNode<T> {
    fn default() -> Self {
        GraphNode {
            id: GraphNodeId::new(),
            index: 0,
            value: Default::default(),
            direction: Direction::Forward,
            node_type: None,
            arity: None,
            incoming: BTreeSet::new(),
            outgoing: BTreeSet::new(),
        }
    }
}

impl<T: Debug> Debug for GraphNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let incoming = self
            .incoming
            .iter()
            .map(|idx| idx.to_string())
            .collect::<Vec<String>>()
            .join(", ");

        write!(
            f,
            "[{:<3}] [{:<7?}] {:>10?} :: {:<10} {:<12} V:{:<5} R:{:<5} {:<2} {:<2} < [{}]",
            self.index,
            self.id.0,
            format!("{:?}", self.node_type())[..3].to_owned(),
            self.arity(),
            format!("{:?}", self.value).to_owned(),
            self.is_valid(),
            self.is_recurrent(),
            self.incoming.len(),
            self.outgoing.len(),
            incoming,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NodeType;
    use std::collections::BTreeSet;

    #[test]
    fn test_graph_node_default() {
        let node = GraphNode::<usize>::default();

        assert_eq!(node.index(), 0);
        assert_eq!(node.node_type(), NodeType::Vertex);
        assert_eq!(node.arity(), Arity::Any);
        assert!(!node.is_valid());
        assert!(!node.is_recurrent());
        assert_eq!(node.incoming(), &BTreeSet::new());
        assert_eq!(node.outgoing(), &BTreeSet::new());
    }

    #[test]
    fn test_graph_node() {
        let node = GraphNode::new(0, NodeType::Input, 0.0);

        assert_eq!(node.index(), 0);
        assert_eq!(node.node_type(), NodeType::Input);
        assert_eq!(node.arity(), Arity::Zero);
        assert!(!node.is_valid());
        assert!(!node.is_recurrent());
        assert_eq!(node.incoming(), &BTreeSet::new());
        assert_eq!(node.outgoing(), &BTreeSet::new());
    }

    #[test]
    fn test_graph_node_with_arity() {
        let node = GraphNode::with_arity(0, NodeType::Input, 0.0, Arity::Zero);

        assert_eq!(node.index(), 0);
        assert_eq!(node.node_type(), NodeType::Input);
        assert_eq!(node.arity(), Arity::Zero);
        assert!(!node.is_valid());
        assert!(!node.is_recurrent());
        assert_eq!(node.incoming(), &BTreeSet::new());
        assert_eq!(node.outgoing(), &BTreeSet::new());
    }

    #[test]
    fn test_graph_node_with_allele() {
        let node = GraphNode::new(0, NodeType::Input, 0.0);

        let new_node = node.with_allele(&1.0);
        assert_eq!(new_node.index(), 0);
        assert_eq!(new_node.node_type(), NodeType::Input);
        assert_eq!(new_node.arity(), Arity::Zero);
        assert!(!new_node.is_valid());
        assert!(!new_node.is_recurrent());
        assert_eq!(new_node.incoming(), &BTreeSet::new());
        assert_eq!(new_node.outgoing(), &BTreeSet::new());
    }

    #[test]
    fn test_graph_node_with_direction() {
        let mut node_one = GraphNode::new(0, NodeType::Input, 0.0);

        assert!(!node_one.is_recurrent());
        node_one.set_direction(Direction::Backward);
        assert!(node_one.is_recurrent());

        let mut node_two = GraphNode::new(0, NodeType::Input, 0.0);

        assert!(!node_two.is_recurrent());
        node_two.incoming_mut().insert(0);
        assert!(node_two.is_recurrent());
    }

    #[test]
    fn graph_node_from_fns_produce_valid_arities() {
        let node = GraphNode::from((0, NodeType::Input, 0.0));
        assert_eq!(node.arity(), Arity::Zero);

        let node = GraphNode::from((0, NodeType::Output, 0.0));
        assert_eq!(node.arity(), Arity::Any);

        let node = GraphNode::from((0, NodeType::Vertex, 0.0));
        assert_eq!(node.arity(), Arity::Any);

        let node = GraphNode::from((0, NodeType::Edge, 0.0));
        assert_eq!(node.arity(), Arity::Exact(1));

        let node = GraphNode::from((0, NodeType::Input, 0.0, Arity::Zero));
        assert_eq!(node.arity(), Arity::Zero);

        let node = GraphNode::from((0, NodeType::Output, 0.0, Arity::Any));
        assert_eq!(node.arity(), Arity::Any);

        let node = GraphNode::from((0, NodeType::Vertex, 0.0, Arity::Any));
        assert_eq!(node.arity(), Arity::Any);

        let node = GraphNode::from((0, NodeType::Edge, 0.0, Arity::Exact(1)));
        assert_eq!(node.arity(), Arity::Exact(1));
    }
}
