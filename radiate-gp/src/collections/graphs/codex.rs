use std::sync::{Arc, RwLock};

use radiate::{Chromosome, Codex, Gene, Genotype};

use crate::Factory;

use super::{Graph, GraphChromosome, GraphNode, NodeStore};

pub struct GraphCodex<T> {
    store: Arc<RwLock<NodeStore<T>>>,
    nodes: Option<Vec<GraphNode<T>>>,
}

impl<T> GraphCodex<T> {
    pub fn new(store: Arc<RwLock<NodeStore<T>>>, nodes: Option<Vec<GraphNode<T>>>) -> Self {
        Self { store, nodes }
    }
}

impl<T> Codex<GraphChromosome<T>, Graph<T>> for GraphCodex<T>
where
    T: Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<GraphChromosome<T>> {
        let store = Arc::clone(&self.store);
        if let Some(nodes) = &self.nodes {
            let new_nodes = nodes
                .iter()
                .map(|node| {
                    let new_node = store
                        .read()
                        .unwrap()
                        .new_instance((node.index(), node.node_type()));

                    if new_node.value().arity() == node.value().arity() {
                        node.with_allele(new_node.allele())
                    } else {
                        node.clone()
                    }
                })
                .collect::<Vec<GraphNode<T>>>();

            let chromosome = GraphChromosome::new(new_nodes, store);
            return Genotype::new(vec![chromosome]);
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
