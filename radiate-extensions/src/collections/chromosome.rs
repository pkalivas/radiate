use crate::collections::{Factory, GraphNode, NodeFactory, NodeType};
use radiate::{Chromosome, Gene, Valid};
use std::cell::RefCell;
use std::ops::{Index, IndexMut};
use std::rc::Rc;
use std::sync::Arc;

type Constraint<N> = Arc<Box<dyn Fn(&N) -> bool>>;

#[derive(Clone, Default)]
pub struct NodeChrom<N>
where
    N: Gene,
{
    nodes: Vec<N>,
    constraint: Option<Constraint<N>>,
}

impl<N> NodeChrom<N>
where
    N: Gene,
{
    pub fn new(nodes: Vec<N>) -> Self {
        NodeChrom {
            nodes,
            constraint: None,
        }
    }

    pub fn with_constraint(nodes: Vec<N>, constraint: Option<Constraint<N>>) -> Self {
        NodeChrom { nodes, constraint }
    }
}

impl<N> Chromosome for NodeChrom<N>
where
    N: Gene,
{
    type Gene = N;

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
    pub nodes: Vec<GraphNode<T>>,
    pub factory: Option<Rc<RefCell<NodeFactory<T>>>>,
}

impl<T> NodeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(nodes: Vec<GraphNode<T>>) -> Self {
        NodeChromosome {
            nodes,
            factory: None,
        }
    }

    pub fn with_factory(nodes: Vec<GraphNode<T>>, factory: Rc<RefCell<NodeFactory<T>>>) -> Self {
        NodeChromosome {
            nodes,
            factory: Some(factory),
        }
    }

    pub fn new_node(&self, index: usize, node_type: NodeType) -> GraphNode<T> {
        let factory = self.factory.as_ref().unwrap();
        let factory = factory.borrow();
        factory.new_instance((index, node_type))
    }
}

impl<T> Chromosome for NodeChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    type Gene = GraphNode<T>;

    fn get_genes(&self) -> &[GraphNode<T>] {
        &self.nodes
    }

    fn get_genes_mut(&mut self) -> &mut [GraphNode<T>] {
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
    type Output = GraphNode<T>;

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
