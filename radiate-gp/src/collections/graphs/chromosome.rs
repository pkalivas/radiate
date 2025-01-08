use crate::{CellStore, GraphNode, NodeCell};
use radiate::{Chromosome, Valid};
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

#[derive(Clone, PartialEq)]
pub struct GraphChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    pub nodes: Vec<GraphNode<C>>,
    pub factory: Option<Rc<RefCell<CellStore<C>>>>,
}

impl<C> GraphChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    pub fn new(nodes: Vec<GraphNode<C>>, factory: Rc<RefCell<CellStore<C>>>) -> Self {
        GraphChromosome {
            nodes,
            factory: Some(factory),
        }
    }
}

impl<C> Chromosome for GraphChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    type Gene = GraphNode<C>;
}

impl<C> Valid for GraphChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    fn is_valid(&self) -> bool {
        self.nodes.iter().all(|gene| gene.is_valid())
    }
}

impl<C> AsRef<[GraphNode<C>]> for GraphChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    fn as_ref(&self) -> &[GraphNode<C>] {
        &self.nodes
    }
}

impl<C> AsMut<[GraphNode<C>]> for GraphChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    fn as_mut(&mut self) -> &mut [GraphNode<C>] {
        &mut self.nodes
    }
}

impl<C> Index<usize> for GraphChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    type Output = GraphNode<C>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.nodes[index]
    }
}

impl<C> IndexMut<usize> for GraphChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.nodes[index]
    }
}

impl<C> Debug for GraphChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Graph {{\n")?;
        for node in self.as_ref() {
            write!(f, "  {:?},\n", node)?;
        }
        write!(f, "}}")
    }
}
