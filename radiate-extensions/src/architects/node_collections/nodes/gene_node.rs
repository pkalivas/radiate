use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use radiate::{Gene, Valid};

use crate::{NodeBehavior, NodeType};

use super::IndexedNode;

#[derive(PartialEq)]
pub struct GeneNode<T> {
    inner: IndexedNode<T>,
    factory: Rc<RefCell<HashMap<NodeType, Vec<T>>>>,
}

impl<T> GeneNode<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(inner: IndexedNode<T>, factory: Rc<RefCell<HashMap<NodeType, Vec<T>>>>) -> Self {
        Self { inner, factory }
    }

    pub fn inner(&self) -> &IndexedNode<T> {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut IndexedNode<T> {
        &mut self.inner
    }

    pub fn factory(&self) -> Rc<RefCell<HashMap<NodeType, Vec<T>>>> {
        self.factory.clone()
    }
}

impl<T> Clone for GeneNode<T>
where
    T: Clone + PartialEq + Default,
{
    fn clone(&self) -> Self {
        let new_inner = self.inner.clone();

        Self {
            inner: new_inner,
            factory: self.factory.clone(),
        }
    }
}

impl<T> Gene for GeneNode<T>
where
    T: Clone + PartialEq + Default,
{
    type Allele = T;

    fn allele(&self) -> &Self::Allele {
        self.inner.node().value()
    }

    fn new_instance(&self) -> Self {
        let node_type = self.inner.node().node_type();
        let values = self.factory.borrow().get(&node_type).unwrap();

        todo!()
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        todo!()
    }
}

impl<T> Valid for GeneNode<T>
where
    T: Clone + PartialEq + Default,
{
    fn is_valid(&self) -> bool {
        todo!()
    }
}
