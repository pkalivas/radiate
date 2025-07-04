use super::{Graph, GraphChromosome, GraphNode};
use crate::{Factory, NodeStore};
use radiate_core::{Chromosome, Codec, Genotype};

#[derive(Clone, Debug)]
pub struct GraphCodec<T> {
    store: NodeStore<T>,
    template: GraphChromosome<T>,
}

impl<T> GraphCodec<T> {
    pub fn directed(input_size: usize, output_size: usize, store: impl Into<NodeStore<T>>) -> Self
    where
        T: Clone + Default,
    {
        let new_store = store.into();

        GraphCodec {
            store: new_store.clone(),
            template: Graph::directed(input_size, output_size, &new_store)
                .into_iter()
                .collect(),
        }
    }

    pub fn recurrent(input_size: usize, output_size: usize, store: impl Into<NodeStore<T>>) -> Self
    where
        T: Clone + Default,
    {
        let new_store = store.into();

        GraphCodec {
            store: new_store.clone(),
            template: Graph::recurrent(input_size, output_size, &new_store)
                .into_iter()
                .collect(),
        }
    }

    pub fn weighted_directed(
        input_size: usize,
        output_size: usize,
        store: impl Into<NodeStore<T>>,
    ) -> Self
    where
        T: Clone + Default,
    {
        let new_store = store.into();

        GraphCodec {
            store: new_store.clone(),
            template: Graph::weighted_directed(input_size, output_size, &new_store)
                .into_iter()
                .collect(),
        }
    }

    pub fn weighted_recurrent(
        input_size: usize,
        output_size: usize,
        store: impl Into<NodeStore<T>>,
    ) -> Self
    where
        T: Clone + Default,
    {
        let new_store = store.into();

        GraphCodec {
            store: new_store.clone(),
            template: Graph::weighted_recurrent(input_size, output_size, &new_store)
                .into_iter()
                .collect(),
        }
    }
}

impl<T> Codec<GraphChromosome<T>, Graph<T>> for GraphCodec<T>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<GraphChromosome<T>> {
        let chromosome = self.template.new_instance(Some(self.store.clone()));
        Genotype::from(chromosome)
    }

    fn decode(&self, genotype: &Genotype<GraphChromosome<T>>) -> Graph<T> {
        Graph::new(
            genotype
                .iter()
                .flat_map(|chrom| chrom.iter())
                .cloned()
                .collect::<Vec<GraphNode<T>>>(),
        )
    }
}
