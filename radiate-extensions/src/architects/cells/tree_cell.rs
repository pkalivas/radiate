use crate::{CellSchema, Direction, Expr, NodeCell};

#[derive(Clone, PartialEq)]
pub struct TreeCell<T> {
    pub inner: Option<NodeCell<T>>,
    pub children: Option<Vec<TreeCell<T>>>,
}

impl<T> TreeCell<T> {
    pub fn new(inner: NodeCell<T>) -> Self {
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

impl<T> CellSchema for TreeCell<T> {
    type ValueType = T;

    fn value(&self) -> &Expr<Self::ValueType> {
        self.inner.as_ref().unwrap().value()
    }

    fn id(&self) -> &uuid::Uuid {
        self.inner.as_ref().unwrap().id()
    }

    fn enabled(&self) -> bool {
        self.inner.as_ref().unwrap().enabled()
    }

    fn direction(&self) -> Direction {
        self.inner.as_ref().unwrap().direction()
    }
}
