use crate::{Factory, GraphNode, NodeFactory, NodeType};
use radiate::{Chromosome, Valid};
use std::cell::RefCell;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

#[derive(Clone)]
pub struct GraphChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    pub nodes: Vec<GraphNode<T>>,
    pub factory: Option<Rc<RefCell<NodeFactory<T>>>>,
}

impl<T> GraphChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(nodes: Vec<GraphNode<T>>) -> Self {
        GraphChromosome {
            nodes,
            factory: None,
        }
    }

    pub fn with_factory(nodes: Vec<GraphNode<T>>, factory: Rc<RefCell<NodeFactory<T>>>) -> Self {
        GraphChromosome {
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

impl<T> Chromosome for GraphChromosome<T>
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

impl<T> Valid for GraphChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    fn is_valid(&self) -> bool {
        self.nodes.iter().all(|gene| gene.is_valid())
    }
}

impl<T> Index<usize> for GraphChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    type Output = GraphNode<T>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<T> IndexMut<usize> for GraphChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

impl<T> PartialEq for GraphChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

impl<T> std::fmt::Debug for GraphChromosome<T>
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
