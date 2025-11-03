use super::{Graph, GraphChromosome, GraphNode};
use crate::{Factory, NodeStore};
use radiate_core::{Chromosome, Codec, Genotype};

#[derive(Clone, Debug)]
pub struct GraphCodec<T> {
    store: NodeStore<T>,
    template: GraphChromosome<T>,
}

impl<T> GraphCodec<T> {
    pub fn new(
        template: impl IntoIterator<Item = GraphNode<T>>,
        store: impl Into<NodeStore<T>>,
    ) -> Self
    where
        T: Clone + Default,
    {
        GraphCodec {
            store: store.into(),
            template: template.into_iter().collect(),
        }
    }

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

    pub fn lstm(input_size: usize, output_size: usize, store: impl Into<NodeStore<T>>) -> Self
    where
        T: Clone + Default,
    {
        let new_store = store.into();

        GraphCodec {
            store: new_store.clone(),
            template: Graph::lstm(input_size, output_size, &new_store)
                .into_iter()
                .collect(),
        }
    }

    pub fn gru(input_size: usize, output_size: usize, store: impl Into<NodeStore<T>>) -> Self
    where
        T: Clone + Default,
    {
        let new_store = store.into();

        GraphCodec {
            store: new_store.clone(),
            template: Graph::gru(input_size, output_size, &new_store)
                .into_iter()
                .collect(),
        }
    }

    pub fn mesh(
        input_size: usize,
        output_size: usize,
        rows: usize,
        cols: usize,
        store: impl Into<NodeStore<T>>,
    ) -> Self
    where
        T: Clone + Default,
    {
        let new_store = store.into();

        GraphCodec {
            store: new_store.clone(),
            template: Graph::mesh(input_size, output_size, rows, cols, &new_store)
                .into_iter()
                .collect(),
        }
    }

    pub fn with_max_nodes(mut self, max_nodes: usize) -> Self {
        self.template = self.template.with_max_nodes(max_nodes);
        self
    }
}

impl<T> Codec<GraphChromosome<T>, Graph<T>> for GraphCodec<T>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<GraphChromosome<T>> {
        self.template.new_instance(Some(self.store.clone())).into()
    }

    #[inline]
    fn decode(&self, genotype: &Genotype<GraphChromosome<T>>) -> Graph<T> {
        let mut new_nodes = Vec::with_capacity(genotype[0].len());
        new_nodes.extend_from_slice(genotype[0].as_ref());
        Graph::new(new_nodes)
    }
}
