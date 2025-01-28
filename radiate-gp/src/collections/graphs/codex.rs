use std::fmt::Debug;

use super::{
    builder::{AsyclicGraphBuilder, CyclicGraphBuilder},
    Graph, GraphChromosome, GraphNode, NodeStore,
};
use crate::{Builder, Factory};
use radiate::{Chromosome, Codex, Gene, Genotype};

pub struct GraphCodex<T> {
    store: NodeStore<T>,
    nodes: Option<Vec<GraphNode<T>>>,
}

impl<T> GraphCodex<T> {
    pub fn asyclic(input_size: usize, output_size: usize, store: impl Into<NodeStore<T>>) -> Self
    where
        T: Clone + Default,
    {
        let new_store = store.into();
        let nodes = AsyclicGraphBuilder::new(input_size, output_size, &new_store)
            .build()
            .into_iter()
            .collect();

        GraphCodex {
            store: new_store,
            nodes: Some(nodes),
        }
    }

    pub fn cyclic(input_size: usize, output_size: usize, store: impl Into<NodeStore<T>>) -> Self
    where
        T: Clone + Default,
    {
        let new_store = store.into();
        let nodes = CyclicGraphBuilder::new(input_size, output_size, &new_store)
            .build()
            .into_iter()
            .collect();

        GraphCodex {
            store: new_store,
            nodes: Some(nodes),
        }
    }
}

impl<T> Codex<GraphChromosome<T>, Graph<T>> for GraphCodex<T>
where
    T: Clone + PartialEq + Default + Debug,
{
    fn encode(&self) -> Genotype<GraphChromosome<T>> {
        let store = self.store.clone();

        if let Some(nodes) = &self.nodes {
            let new_nodes = nodes
                .iter()
                .map(|node| {
                    let new_node = self.store.new_instance((node.index(), node.node_type()));

                    if new_node.arity() == node.arity() {
                        node.with_allele(new_node.allele())
                    } else {
                        node.clone()
                    }
                })
                .collect::<Vec<GraphNode<T>>>();

            return Genotype::new(vec![GraphChromosome::new(new_nodes, store)]);
        }

        panic!("GraphBuilder has no nodes to encode");
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
