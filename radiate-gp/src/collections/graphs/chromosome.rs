use super::CellStore;
use crate::{GraphNode, NodeCell};
use radiate::{Chromosome, Valid};
use std::fmt::Debug;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct GraphChromosome<T> {
    nodes: Vec<GraphNode<T>>,
    store: Option<Arc<RwLock<CellStore<T>>>>,
}

impl<T> GraphChromosome<T> {
    pub fn new(nodes: Vec<GraphNode<T>>, factory: Arc<RwLock<CellStore<T>>>) -> Self {
        GraphChromosome {
            nodes,
            store: Some(factory),
        }
    }

    pub fn set_nodes(&mut self, nodes: Vec<GraphNode<T>>) {
        self.nodes = nodes;
    }

    pub fn store(&self) -> Arc<RwLock<CellStore<T>>> {
        self.store.as_ref().unwrap().clone()
    }
}

impl<T> Chromosome for GraphChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    type Gene = GraphNode<T>;
}

impl<T> Valid for GraphChromosome<T> {
    fn is_valid(&self) -> bool {
        self.nodes.iter().all(|gene| gene.is_valid())
    }
}

impl<T> AsRef<[GraphNode<T>]> for GraphChromosome<T> {
    fn as_ref(&self) -> &[GraphNode<T>] {
        &self.nodes
    }
}

impl<T> AsMut<[GraphNode<T>]> for GraphChromosome<T> {
    fn as_mut(&mut self) -> &mut [GraphNode<T>] {
        &mut self.nodes
    }
}

impl<T: PartialEq> PartialEq for GraphChromosome<T> {
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
