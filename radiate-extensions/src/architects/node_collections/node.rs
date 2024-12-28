use crate::architects::cells::expr::Expr;
use crate::architects::schema::{direction::Direction, node_types::NodeType};
use crate::schema::collection_type::CollectionType;
use crate::{GraphCell, IndexedCell, NodeCell, TreeCell, ValueCell};
use radiate::engines::genome::genes::gene::{Gene, Valid};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Clone, PartialEq)]
pub struct NodeTwo<T> {
    pub inner: NodeCell<T>,
    pub id: Uuid,
    pub node_type: NodeType,
}

impl<T> NodeTwo<T> {
    pub fn new(inner: NodeCell<T>, node_type: NodeType) -> Self {
        NodeTwo {
            inner,
            id: Uuid::new_v4(),
            node_type,
        }
    }

    pub fn collection_type(&self) -> &CollectionType {
        match &self.inner {
            NodeCell::Tree(_) => &CollectionType::Tree,
            NodeCell::FlatTree(_) => &CollectionType::Tree,
            NodeCell::Graph(_) => &CollectionType::Graph,
        }
    }
}

impl<T> Gene for NodeTwo<T>
where
    T: Clone + PartialEq + Default,
{
    type Allele = Expr<T>;

    fn allele(&self) -> &Expr<T> {
        self.inner.as_ref().value()
    }

    fn new_instance(&self) -> Self {
        NodeTwo {
            inner: match &self.inner {
                NodeCell::Tree(cell) => NodeCell::Tree(cell.clone()),
                NodeCell::FlatTree(cell) => NodeCell::FlatTree(IndexedCell {
                    inner: ValueCell::new(cell.inner.value.new_instance()),
                    index: cell.index,
                    incoming: cell.incoming.clone(),
                    outgoing: cell.outgoing.clone(),
                }),
                NodeCell::Graph(cell) => NodeCell::Graph(GraphCell {
                    inner: IndexedCell {
                        inner: ValueCell::new(cell.inner.inner.value.new_instance()),
                        index: cell.inner.index,
                        incoming: cell.inner.incoming.clone(),
                        outgoing: cell.inner.outgoing.clone(),
                    },
                    enabled: cell.enabled,
                    direction: cell.direction,
                }),
            },
            id: uuid::Uuid::new_v4(),
            node_type: self.node_type,
        }
    }

    fn with_allele(&self, allele: &Expr<T>) -> Self {
        NodeTwo {
            inner: match &self.inner {
                NodeCell::Tree(cell) => NodeCell::Tree(TreeCell {
                    inner: Some(ValueCell::new(allele.clone())),
                    children: cell.children.clone(),
                }),
                NodeCell::FlatTree(cell) => NodeCell::FlatTree(IndexedCell {
                    inner: ValueCell::new(allele.clone()),
                    index: cell.index,
                    incoming: cell.incoming.clone(),
                    outgoing: cell.outgoing.clone(),
                }),
                NodeCell::Graph(cell) => NodeCell::Graph(GraphCell {
                    inner: IndexedCell {
                        inner: ValueCell::new(allele.clone()),
                        index: cell.inner.index,
                        incoming: cell.inner.incoming.clone(),
                        outgoing: cell.inner.outgoing.clone(),
                    },
                    enabled: cell.enabled,
                    direction: cell.direction,
                }),
            },
            id: uuid::Uuid::new_v4(),
            node_type: self.node_type,
        }
    }
}

impl<T> Valid for NodeTwo<T>
where
    T: Clone + PartialEq + Default,
{
    fn is_valid(&self) -> bool {
        match &self.inner {
            NodeCell::Tree(_) => true,
            NodeCell::FlatTree(cell) => match self.node_type {
                NodeType::Input => cell.incoming.is_empty() && !cell.outgoing.is_empty(),
                NodeType::Output => !cell.incoming.is_empty(),
                NodeType::Gate => cell.outgoing.len() == cell.inner.value.arity() as usize,
                NodeType::Aggregate => !cell.incoming.is_empty() && !cell.outgoing.is_empty(),
                NodeType::Weight => cell.incoming.len() == 1 && cell.outgoing.len() == 1,
                NodeType::Link => cell.incoming.len() == 1 && !cell.outgoing.is_empty(),
                NodeType::Leaf => !cell.incoming.is_empty() && cell.outgoing.is_empty(),
            },
            NodeCell::Graph(cell) => match self.node_type {
                NodeType::Input => {
                    cell.inner.incoming.is_empty() && !cell.inner.outgoing.is_empty()
                }
                NodeType::Output => !cell.inner.incoming.is_empty(),
                NodeType::Gate => {
                    cell.inner.outgoing.len() == cell.inner.inner.value.arity() as usize
                }
                NodeType::Aggregate => {
                    !cell.inner.incoming.is_empty() && !cell.inner.outgoing.is_empty()
                }
                NodeType::Weight => {
                    cell.inner.incoming.len() == 1 && cell.inner.outgoing.len() == 1
                }
                NodeType::Link => cell.inner.incoming.len() == 1 && !cell.inner.outgoing.is_empty(),
                NodeType::Leaf => cell.inner.incoming.is_empty() && !cell.inner.outgoing.is_empty(),
            },
        }
    }
}

////////////////////////////////////////
/// OLD CODE
/// ////////////////////////////////////////

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
                    NodeType::Gate => self.incoming.len() == self.value.arity() as usize,
                    NodeType::Aggregate => !self.incoming.is_empty() && !self.outgoing.is_empty(),
                    NodeType::Weight => self.incoming.len() == 1 && self.outgoing.len() == 1,
                    NodeType::Link => self.incoming.len() == 1 && !self.outgoing.is_empty(),
                    NodeType::Leaf => self.incoming.is_empty() && !self.outgoing.is_empty(),
                };
            } else if coll_type == &CollectionType::Tree {
                return match self.node_type {
                    NodeType::Input => self.incoming.is_empty() && !self.outgoing.is_empty(),
                    NodeType::Output => !self.incoming.is_empty(),
                    NodeType::Gate => self.outgoing.len() == self.value.arity() as usize,
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
    T: Clone + PartialEq,
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
