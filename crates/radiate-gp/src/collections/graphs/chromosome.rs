use crate::{Factory, GraphNode, NodeStore, node::Node};
use radiate_core::{Chromosome, Gene, Valid};
use std::fmt::Debug;

#[derive(Clone)]
pub struct GraphChromosome<T> {
    nodes: Vec<GraphNode<T>>,
    store: Option<NodeStore<T>>,
}

impl<T> GraphChromosome<T> {
    pub fn new(nodes: Vec<GraphNode<T>>, factory: NodeStore<T>) -> Self {
        GraphChromosome {
            nodes,
            store: Some(factory),
        }
    }

    pub fn set_nodes(&mut self, nodes: Vec<GraphNode<T>>) {
        self.nodes = nodes;
    }

    pub fn store(&self) -> Option<&NodeStore<T>> {
        self.store.as_ref()
    }
}

impl<T> Factory<Option<NodeStore<T>>, GraphChromosome<T>> for GraphChromosome<T>
where
    T: Clone + PartialEq + Default,
{
    fn new_instance(&self, input: Option<NodeStore<T>>) -> GraphChromosome<T> {
        let maybe_store = input.or_else(|| self.store.clone());
        if let Some(store) = maybe_store {
            return GraphChromosome {
                nodes: self
                    .iter()
                    .enumerate()
                    .map(|(index, node)| {
                        let new_node = store.new_instance((index, node.node_type()));
                        if new_node.arity() == node.arity() {
                            node.with_allele(new_node.allele())
                        } else {
                            node.clone()
                        }
                    })
                    .collect(),
                store: Some(store),
            };
        }

        self.clone()
    }
}

impl<T> Chromosome for GraphChromosome<T>
where
    T: Clone + PartialEq,
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

impl<T> FromIterator<GraphNode<T>> for GraphChromosome<T> {
    fn from_iter<I: IntoIterator<Item = GraphNode<T>>>(iter: I) -> Self {
        GraphChromosome {
            nodes: iter.into_iter().collect(),
            store: None,
        }
    }
}

impl<T: Debug> Debug for GraphChromosome<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Graph {{\n")?;
        for node in self.as_ref() {
            write!(f, "  {:?},\n", node)?;
        }
        write!(f, "}}")
    }
}
