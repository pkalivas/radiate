use crate::{GraphNode, NodeFactory};
use radiate::{Chromosome, Valid};
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

#[derive(Clone, PartialEq)]
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
    pub fn new(nodes: Vec<GraphNode<T>>, factory: Rc<RefCell<NodeFactory<T>>>) -> Self {
        GraphChromosome {
            nodes,
            factory: Some(factory),
        }
    }
}

impl<T> Chromosome for GraphChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    type Gene = GraphNode<T>;
}

impl<T> Valid for GraphChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    fn is_valid(&self) -> bool {
        self.nodes.iter().all(|gene| gene.is_valid())
    }
}

impl<T> AsRef<[GraphNode<T>]> for GraphChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    fn as_ref(&self) -> &[GraphNode<T>] {
        &self.nodes
    }
}

impl<T> AsMut<[GraphNode<T>]> for GraphChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    fn as_mut(&mut self) -> &mut [GraphNode<T>] {
        &mut self.nodes
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

impl<T> Debug for GraphChromosome<T>
where
    T: Clone + PartialEq + Default + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Graph {{\n")?;
        for node in self.as_ref() {
            write!(f, "  {:?},\n", node)?;
        }
        write!(f, "}}")
    }
}
