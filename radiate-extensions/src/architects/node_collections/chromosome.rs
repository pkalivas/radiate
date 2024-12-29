use crate::{Node, NodeFactory, NodeType};
use radiate::{Chromosome, Gene, Valid};
use std::cell::RefCell;
use std::ops::{Index, IndexMut};
use std::rc::Rc;
use std::sync::Arc;

type AlleleStore<T> = Option<Arc<Box<Vec<T>>>>;

#[derive(Clone, Default)]
pub struct NodeChrom<N>
where
    N: Gene,
    N::Allele: Clone,
{
    nodes: Vec<N>,
    constraint: Option<Arc<Box<dyn Fn(&N) -> bool>>>,
    providers: AlleleStore<N::Allele>,
    internals: AlleleStore<N::Allele>,
    outputs: AlleleStore<N::Allele>,
}

impl<N> NodeChrom<N>
where
    N: Gene,
    N::Allele: Clone,
{
    pub fn new(nodes: Vec<N>) -> Self {
        NodeChrom {
            nodes,
            constraint: None,
            providers: None,
            internals: None,
            outputs: None,
        }
    }

    pub fn with_constraint(
        nodes: Vec<N>,
        constraint: Option<Arc<Box<dyn Fn(&N) -> bool>>>,
    ) -> Self {
        NodeChrom {
            nodes,
            constraint,
            providers: None,
            internals: None,
            outputs: None,
        }
    }

    pub fn set_providers(&mut self, providers: Arc<Box<Vec<N::Allele>>>) {
        self.providers = Some(Arc::clone(&providers));
    }

    pub fn set_internals(&mut self, internals: Arc<Box<Vec<N::Allele>>>) {
        self.internals = Some(Arc::clone(&internals));
    }

    pub fn set_outputs(&mut self, outputs: Arc<Box<Vec<N::Allele>>>) {
        self.outputs = Some(Arc::clone(&outputs));
    }

    pub fn get_providers(&self) -> Option<&Vec<N::Allele>> {
        self.providers.as_ref().map(|p| p.as_ref().as_ref())
    }

    pub fn get_internals(&self) -> Option<&Vec<N::Allele>> {
        self.internals.as_ref().map(|p| p.as_ref().as_ref())
    }

    pub fn get_outputs(&self) -> Option<&Vec<N::Allele>> {
        self.outputs.as_ref().map(|p| p.as_ref().as_ref())
    }
}

impl<N> Chromosome for NodeChrom<N>
where
    N: Gene,
    N::Allele: Clone,
{
    type Gene = N;

    fn from_genes(genes: Vec<N>) -> Self {
        NodeChrom {
            nodes: genes,
            constraint: None,
            providers: None,
            internals: None,
            outputs: None,
        }
    }

    fn get_genes(&self) -> &[N] {
        &self.nodes
    }

    fn get_genes_mut(&mut self) -> &mut [N] {
        &mut self.nodes
    }
}

impl<N> Valid for NodeChrom<N>
where
    N: Gene,
    N::Allele: Clone,
{
    fn is_valid(&self) -> bool {
        for gene in &self.nodes {
            if let Some(constraint) = &self.constraint {
                if !constraint(gene) {
                    return false;
                }
            } else if !gene.is_valid() {
                return false;
            }
        }

        true
    }
}

impl<N> PartialEq for NodeChrom<N>
where
    N: Gene,
    N::Allele: Clone,
{
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

#[derive(Clone)]
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
        panic!("Not implemented")
        // factory.new_node(index, node_type)
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

impl<T> std::fmt::Debug for NodeChromosome<T>
where
    T: Clone + PartialEq + Default + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Graph {{\n")?;
        for node in self.get_genes() {
            write!(f, "  {:?},\n", node)?;
        }
        write!(f, "}}")
    }
}
