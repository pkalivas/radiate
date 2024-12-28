pub mod expr;

use std::collections::HashSet;

pub use expr::*;

use super::Direction;

#[derive(Clone, PartialEq)]
pub struct ValueCell<T> {
    pub value: Expr<T>,
}

impl<T> ValueCell<T> {
    pub fn new(value: Expr<T>) -> Self {
        ValueCell { value }
    }

    pub fn value(&self) -> &Expr<T> {
        &self.value
    }

    pub fn value_mut(&mut self) -> &mut Expr<T> {
        &mut self.value
    }
}

impl<T: Default> Default for ValueCell<T> {
    fn default() -> Self {
        ValueCell {
            value: Expr::default(),
        }
    }
}

impl<T> From<Expr<T>> for ValueCell<T> {
    fn from(value: Expr<T>) -> Self {
        ValueCell { value }
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

#[derive(Clone, PartialEq, Default)]
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

impl<T> From<ValueCell<T>> for IndexedCell<T> {
    fn from(inner: ValueCell<T>) -> Self {
        IndexedCell {
            inner,
            index: 0,
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
