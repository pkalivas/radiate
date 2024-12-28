pub mod expr;

use std::{collections::HashSet, rc::Rc};

use expr::Expr;
use radiate::{Gene, Valid};

use super::{Direction, NodeType};

type ValuePermutations<T> = Option<Rc<Vec<Expr<T>>>>;

#[derive(Clone, PartialEq)]
pub struct ValueCell<T> {
    pub value: Expr<T>,
    pub permutations: ValuePermutations<T>,
}

impl<T> ValueCell<T> {
    pub fn new(value: Expr<T>) -> Self {
        ValueCell {
            value,
            permutations: None,
        }
    }

    pub fn with_permutations(value: Expr<T>, permutations: ValuePermutations<T>) -> Self {
        ValueCell {
            value,
            permutations,
        }
    }

    pub fn value(&self) -> &Expr<T> {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut Expr<T> {
        &mut self.value
    }
}

impl<T> AsRef<ValueCell<T>> for ValueCell<T> {
    fn as_ref(&self) -> &ValueCell<T> {
        self
    }
}

impl<T> AsMut<ValueCell<T>> for ValueCell<T> {
    fn as_mut(&mut self) -> &mut ValueCell<T> {
        self
    }
}

#[derive(Clone, PartialEq)]
pub struct IndexedCell<T> {
    pub inner: ValueCell<T>,
    pub index: usize,
    pub incoming: HashSet<usize>,
    pub outgoing: HashSet<usize>,
}

impl<T> IndexedCell<T> {
    pub fn new(inner: ValueCell<T>, index: usize) -> Self {
        IndexedCell {
            inner,
            index,
            incoming: HashSet::new(),
            outgoing: HashSet::new(),
        }
    }
}

impl<T> AsRef<ValueCell<T>> for IndexedCell<T> {
    fn as_ref(&self) -> &ValueCell<T> {
        &self.inner
    }
}

impl<T> AsMut<ValueCell<T>> for IndexedCell<T> {
    fn as_mut(&mut self) -> &mut ValueCell<T> {
        &mut self.inner
    }
}

#[derive(Clone, PartialEq)]
pub struct TreeCell<T> {
    pub inner: Option<ValueCell<T>>,
    pub children: Option<Vec<TreeCell<T>>>,
}

impl<T> TreeCell<T> {
    pub fn new(inner: ValueCell<T>) -> Self {
        TreeCell {
            inner: Some(inner),
            children: None,
        }
    }

    pub fn add_child(&mut self, child: TreeCell<T>) {
        if self.children.is_none() {
            self.children = Some(vec![]);
        }

        self.children.as_mut().unwrap().push(child);
    }

    pub fn children(&self) -> Option<&Vec<TreeCell<T>>> {
        self.children.as_ref()
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<TreeCell<T>>> {
        self.children.as_mut()
    }
}

impl<T> AsRef<ValueCell<T>> for TreeCell<T> {
    fn as_ref(&self) -> &ValueCell<T> {
        self.inner.as_ref().unwrap()
    }
}

impl<T> AsMut<ValueCell<T>> for TreeCell<T> {
    fn as_mut(&mut self) -> &mut ValueCell<T> {
        self.inner.as_mut().unwrap()
    }
}

#[derive(Clone, PartialEq)]
pub struct GraphCell<T> {
    pub inner: IndexedCell<T>,
    pub enabled: bool,
    pub direction: Direction,
}

impl<T> GraphCell<T> {
    pub fn new(inner: IndexedCell<T>) -> Self {
        GraphCell {
            inner,
            enabled: true,
            direction: Direction::Forward,
        }
    }
}

impl<T> AsRef<ValueCell<T>> for GraphCell<T> {
    fn as_ref(&self) -> &ValueCell<T> {
        self.inner.as_ref()
    }
}

impl<T> AsMut<ValueCell<T>> for GraphCell<T> {
    fn as_mut(&mut self) -> &mut ValueCell<T> {
        self.inner.as_mut()
    }
}

#[derive(Clone, PartialEq)]
pub enum NodeCell<T> {
    Tree(TreeCell<T>),
    FlatTree(IndexedCell<T>),
    Graph(GraphCell<T>),
}

impl<T> AsRef<ValueCell<T>> for NodeCell<T> {
    fn as_ref(&self) -> &ValueCell<T> {
        match self {
            NodeCell::Tree(cell) => cell.as_ref(),
            NodeCell::FlatTree(cell) => cell.as_ref(),
            NodeCell::Graph(cell) => cell.as_ref(),
        }
    }
}

impl<T> AsMut<ValueCell<T>> for NodeCell<T> {
    fn as_mut(&mut self) -> &mut ValueCell<T> {
        match self {
            NodeCell::Tree(cell) => cell.as_mut(),
            NodeCell::FlatTree(cell) => cell.as_mut(),
            NodeCell::Graph(cell) => cell.as_mut(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct NodeTwo<T> {
    pub inner: NodeCell<T>,
    pub node_type: NodeType,
}

impl<T> NodeTwo<T> {
    pub fn new(inner: NodeCell<T>, node_type: NodeType) -> Self {
        NodeTwo { inner, node_type }
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
                NodeCell::FlatTree(cell) => NodeCell::FlatTree(cell.clone()),
                NodeCell::Graph(cell) => NodeCell::Graph(cell.clone()),
            },
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
