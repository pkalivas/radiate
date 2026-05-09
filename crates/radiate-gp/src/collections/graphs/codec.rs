use super::{Graph, GraphChromosome, GraphNode};
use crate::node::Node;
use crate::{Factory, NodeStore, NodeType};
use radiate_core::{Codec, Freezable, Freeze, Frozen, Genotype};

#[derive(Clone, Freeze)]
pub struct GraphCodec<T> {
    #[freeze(skip)]
    store: NodeStore<T>,
    #[freeze(with = "test_freeze")]
    template: GraphChromosome<T>,
}

pub fn test_freeze<T>(chromosome: &GraphChromosome<T>) -> Frozen {
    let template = chromosome.as_ref();
    let count_by = |t: NodeType| template.iter().filter(|n| n.node_type() == t).count();
    Frozen::typed::<GraphCodec<T>>()
        .with("aaaaaaaa", count_by(NodeType::Input))
        .with("output_size", count_by(NodeType::Output))
        .with("vertex_size", count_by(NodeType::Vertex))
        .with("edge_size", count_by(NodeType::Edge))
        .with("template_size", template.len())
}

impl<T: Clone + Default> GraphCodec<T> {
    pub fn new(
        template: impl IntoIterator<Item = GraphNode<T>>,
        store: impl Into<NodeStore<T>>,
    ) -> Self {
        GraphCodec {
            store: store.into(),
            template: template.into_iter().collect(),
        }
    }

    pub fn directed(input_size: usize, output_size: usize, store: impl Into<NodeStore<T>>) -> Self {
        let new_store = store.into();

        GraphCodec {
            store: new_store.clone(),
            template: Graph::directed(input_size, output_size, &new_store)
                .into_iter()
                .collect(),
        }
    }

    pub fn recurrent(
        input_size: usize,
        output_size: usize,
        store: impl Into<NodeStore<T>>,
    ) -> Self {
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
    ) -> Self {
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
    ) -> Self {
        let new_store = store.into();

        GraphCodec {
            store: new_store.clone(),
            template: Graph::weighted_recurrent(input_size, output_size, &new_store)
                .into_iter()
                .collect(),
        }
    }

    pub fn lstm(input_size: usize, output_size: usize, store: impl Into<NodeStore<T>>) -> Self {
        let new_store = store.into();

        GraphCodec {
            store: new_store.clone(),
            template: Graph::lstm(input_size, output_size, &new_store)
                .into_iter()
                .collect(),
        }
    }

    pub fn gru(input_size: usize, output_size: usize, store: impl Into<NodeStore<T>>) -> Self {
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
    ) -> Self {
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
        Graph::new(genotype[0].as_ref().to_vec())
    }

    fn freeze(&self) -> Frozen {
        <Self as Freezable>::freeze(self)

        // let template = self.template.as_ref();
        // let count_by = |t: NodeType| template.iter().filter(|n| n.node_type() == t).count();
        // let f = Frozen::typed::<Self>()
        //     .with("input_size", count_by(NodeType::Input))
        //     .with("output_size", count_by(NodeType::Output))
        //     .with("vertex_size", count_by(NodeType::Vertex))
        //     .with("edge_size", count_by(NodeType::Edge))
        //     .with("template_size", template.len());
        // match self.template.max_nodes() {
        //     Some(max) => f.with("max_nodes", max),
        //     None => f,
        // }
    }
}
