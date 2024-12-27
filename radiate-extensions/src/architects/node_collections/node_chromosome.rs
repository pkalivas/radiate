use crate::{Node, NodeFactory, NodeType};
use radiate::{Chromosome, Valid};
use std::cell::RefCell;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct NodeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    pub nodes: Vec<Node<T>>,
    pub factory: Option<Rc<RefCell<NodeFactory<T>>>>,
}

impl<T> NodeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(nodes: Vec<Node<T>>) -> Self {
        NodeChromosome {
            nodes,
            factory: None,
        }
    }

    pub fn with_factory(nodes: Vec<Node<T>>, factory: Rc<RefCell<NodeFactory<T>>>) -> Self {
        NodeChromosome {
            nodes,
            factory: Some(factory),
        }
    }

    pub fn new_node(&self, index: usize, node_type: NodeType) -> Node<T> {
        let factory = self.factory.as_ref().unwrap();
        let factory = factory.borrow();
        factory.new_node(index, node_type)
    }
}

impl<T> Chromosome for NodeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    type Gene = Node<T>;

    fn from_genes(genes: Vec<Node<T>>) -> Self {
        NodeChromosome {
            nodes: genes,
            factory: None,
        }
    }

    fn get_genes(&self) -> &[Node<T>] {
        &self.nodes
    }

    fn get_genes_mut(&mut self) -> &mut [Node<T>] {
        &mut self.nodes
    }
}

impl<T> Valid for NodeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    fn is_valid(&self) -> bool {
        self.nodes.iter().all(|gene| gene.is_valid())
    }
}

impl<T> Index<usize> for NodeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    type Output = Node<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<T> IndexMut<usize> for NodeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

impl<T> PartialEq for NodeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}
