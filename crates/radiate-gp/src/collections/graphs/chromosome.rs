use crate::{Factory, GraphNode, NodeStore, node::Node};
use radiate_core::{Chromosome, Gene, Valid};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// A chromosome type that represents a directed graph structure for genetic programming.
/// This chromosome is essentially just a graph, the only difference is the name of the struct.
/// The graph and the [GraphChromosome] are interchangeable, the only difference is who holds
/// the vector of nodes. For instance, to create a graph from the [GraphChromosome], just take
/// the vector of nodes and give them to a new graph instance - boom, you have a graph.
///
/// [GraphChromosome] is a specialized chromosome type that maintains a collection of graph nodes
/// and their connections. It's designed for genetic programming applications where the solution
/// space can be represented as a directed graph.
///
/// # Type Parameters
/// * `T` - The type of value stored in each node. Must implement `Clone` and `PartialEq`.
///
/// # Structure
/// The chromosome consists of:
/// * A vector of [`GraphNode<T>`] instances representing the graph structure
/// * An optional [`NodeStore<T>`] for managing node creation and validation. This makes
/// the creation of new nodes easier and more uniform across the genetic algorithm.
///
/// # Features
/// * Maintains graph connectivity through node connections
/// * Provides factory methods for creating new instances
/// * Implements serialization when the "serde" feature is enabled
/// * Allows the graph nodes, essentially the graph, to be evolved through the genetic algorithm
///
/// # Examples
/// ```
/// use radiate_gp::collections::graphs::{GraphChromosome, GraphNode, Graph};
/// use radiate_gp::{NodeStore, node_store, NodeType};
///
/// // Create a new chromosome with some nodes
///let store = node_store! {
///     Input => vec![1, 2, 3],
///     Output => vec![4, 5, 6],
///     Edge => vec![7, 8, 9],
///     Vertex => vec![10, 11, 12]
/// };
///
/// let graph = Graph::directed(1, 1, store.clone());
///
/// let chromosome = GraphChromosome::from((graph, store));
/// ```
///
/// # Genetic Operations
/// The chromosome supports several genetic operations:
/// * Crossover through `GraphCrossover`
/// * Mutation through `GraphMutator`
/// * Replacement through `GraphReplacement`
///
/// # Serialization
/// When the "serde" feature is enabled, the chromosome can be serialized and deserialized.
/// The serialization preserves the graph structure and node values, but not the node store.
///
/// # Performance
/// * Node access is O(1) through vector indexing
/// * Graph operations (adding/removing nodes/edges) are O(log n) due to BTreeSet usage
/// * Memory usage is O(V + E) where V is the number of nodes and E is the number of edges
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphChromosome<T> {
    nodes: Vec<GraphNode<T>>,
    store: Option<NodeStore<T>>,
    max_nodes: Option<usize>,
}

impl<T> GraphChromosome<T> {
    pub fn new(nodes: Vec<GraphNode<T>>, factory: NodeStore<T>) -> Self {
        GraphChromosome {
            nodes,
            store: Some(factory),
            max_nodes: None,
        }
    }

    pub fn with_max_nodes(mut self, max_nodes: usize) -> Self {
        self.max_nodes = Some(max_nodes + self.nodes.len());
        self
    }

    pub fn take_nodes(&mut self) -> Vec<GraphNode<T>> {
        std::mem::take(&mut self.nodes)
    }

    pub fn set_nodes(&mut self, nodes: Vec<GraphNode<T>>) {
        self.nodes = nodes;
    }

    pub fn store(&self) -> Option<&NodeStore<T>> {
        self.store.as_ref()
    }

    pub fn max_nodes(&self) -> Option<usize> {
        self.max_nodes
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
                max_nodes: self.max_nodes,
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

    fn genes(&self) -> &[GraphNode<T>] {
        &self.nodes
    }

    fn genes_mut(&mut self) -> &mut [GraphNode<T>] {
        &mut self.nodes
    }
}

impl<T> Valid for GraphChromosome<T> {
    #[inline]
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

impl<T> From<Vec<GraphNode<T>>> for GraphChromosome<T> {
    fn from(nodes: Vec<GraphNode<T>>) -> Self {
        GraphChromosome {
            nodes,
            store: None,
            max_nodes: None,
        }
    }
}

impl<T, I> From<(I, NodeStore<T>)> for GraphChromosome<T>
where
    I: IntoIterator<Item = GraphNode<T>>,
{
    fn from((iter, store): (I, NodeStore<T>)) -> Self {
        GraphChromosome {
            nodes: iter.into_iter().collect(),
            store: Some(store),
            max_nodes: None,
        }
    }
}

impl<T> FromIterator<GraphNode<T>> for GraphChromosome<T> {
    fn from_iter<I: IntoIterator<Item = GraphNode<T>>>(iter: I) -> Self {
        GraphChromosome {
            nodes: iter.into_iter().collect(),
            store: None,
            max_nodes: None,
        }
    }
}

impl<T> IntoIterator for GraphChromosome<T> {
    type Item = GraphNode<T>;
    type IntoIter = std::vec::IntoIter<GraphNode<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.into_iter()
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
