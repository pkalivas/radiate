use super::CellStore;
use crate::{GraphNode, NodeCell};
use radiate::{Chromosome, Valid};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct GraphChromosome<C: NodeCell> {
    nodes: Vec<GraphNode<C>>,
    store: Option<Arc<RwLock<CellStore<C>>>>,
}

impl<C: NodeCell> GraphChromosome<C> {
    pub fn new(nodes: Vec<GraphNode<C>>, factory: Arc<RwLock<CellStore<C>>>) -> Self {
        GraphChromosome {
            nodes,
            store: Some(factory),
        }
    }

    pub fn set_nodes(&mut self, nodes: Vec<GraphNode<C>>) {
        self.nodes = nodes;
    }

    pub fn store(&self) -> Arc<RwLock<CellStore<C>>> {
        self.store.as_ref().unwrap().clone()
    }
}

impl<C> Chromosome for GraphChromosome<C>
where
    C: Clone + PartialEq + Default + NodeCell,
{
    type Gene = GraphNode<C>;
}

impl<C: NodeCell> Valid for GraphChromosome<C> {
    fn is_valid(&self) -> bool {
        self.nodes.iter().all(|gene| gene.is_valid())
    }
}

impl<C: NodeCell> AsRef<[GraphNode<C>]> for GraphChromosome<C> {
    fn as_ref(&self) -> &[GraphNode<C>] {
        &self.nodes
    }
}

impl<C: NodeCell> AsMut<[GraphNode<C>]> for GraphChromosome<C> {
    fn as_mut(&mut self) -> &mut [GraphNode<C>] {
        &mut self.nodes
    }
}

impl<C: NodeCell + PartialEq> PartialEq for GraphChromosome<C> {
    fn eq(&self, other: &Self) -> bool {
        self.nodes == other.nodes
    }
}

impl<C> Debug for GraphChromosome<C>
where
    C: Clone + PartialEq + NodeCell + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Graph {{\n")?;
        for node in self.as_ref() {
            write!(f, "  {:?},\n", node)?;
        }
        write!(f, "}}")
    }
}
