use crate::{CellStore, GraphNode, NodeCell};
use radiate::{Chromosome, Valid};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct GraphChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    pub nodes: Vec<GraphNode<C>>,
    pub store: Option<Arc<RwLock<CellStore<C>>>>,
}

impl<C> GraphChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    pub fn new(nodes: Vec<GraphNode<C>>, factory: Arc<RwLock<CellStore<C>>>) -> Self {
        GraphChromosome {
            nodes,
            store: Some(factory),
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

impl<C> PartialEq for GraphChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}
