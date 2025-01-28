use super::{Graph, GraphChromosome, GraphNode, NodeStore};
use crate::Factory;
use radiate::{Chromosome, Codex, Genotype};
use std::fmt::Debug;

pub struct GraphCodex<T> {
    store: NodeStore<T>,
    template: GraphChromosome<T>,
}

impl<T> GraphCodex<T> {
    pub fn asyclic(input_size: usize, output_size: usize, store: impl Into<NodeStore<T>>) -> Self
    where
        T: Clone + Default,
    {
        let new_store = store.into();

        GraphCodex {
            store: new_store.clone(),
            template: Graph::directed(input_size, output_size, &new_store)
                .into_iter()
                .collect(),
        }
    }

    pub fn cyclic(input_size: usize, output_size: usize, store: impl Into<NodeStore<T>>) -> Self
    where
        T: Clone + Default,
    {
        let new_store = store.into();

        GraphCodex {
            store: new_store.clone(),
            template: Graph::recurrent(input_size, output_size, &new_store)
                .into_iter()
                .collect(),
        }
    }
}

impl<T> Codex<GraphChromosome<T>, Graph<T>> for GraphCodex<T>
where
    T: Clone + PartialEq + Default + Debug,
{
    fn encode(&self) -> Genotype<GraphChromosome<T>> {
        let chromosome = self.template.new_instance(Some(self.store.clone()));
        return Genotype::new(vec![chromosome]);
    }

    fn decode(&self, genotype: &Genotype<GraphChromosome<T>>) -> Graph<T> {
        Graph::new(
            genotype
                .iter()
                .next()
                .unwrap()
                .iter()
                .cloned()
                .collect::<Vec<GraphNode<T>>>(),
        )
    }
}
