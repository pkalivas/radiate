use super::transaction::{InsertStep, TransactionResult};
use super::{Graph, GraphChromosome};
use crate::node::Node;
use crate::{Arity, Factory, NodeType};
use radiate_core::Chromosome;
use radiate_core::{AlterResult, Mutate, metric, random_provider};
use smallvec::SmallVec;

const INVALID_MUTATION: &str = "GraphMutator(Ivld)";

/// A graph mutator that can be used to alter the graph structure within a [`GraphChromosome<T>`].
/// By adding new vertices and edges to the graph, it can be used to explore the search space of a graph.
///
/// # Arguments
/// - `vertex_rate`: The probability of adding a vertex.
/// - `edge_rate`: The probability of adding an edge.
/// - `allow_recurrent`: If true, recurrent nodes are allowed. If false, they are not.
#[derive(Clone, Debug)]
pub struct GraphMutator {
    vertex_rate: f32,
    edge_rate: f32,
    allow_recurrent: bool,
}

// updated GraphMutator implementation
impl GraphMutator {
    /// Create a new graph mutator with a set of mutations
    ///
    /// # Arguments
    /// - `vertex_rate`: The probability of adding a vertex.
    /// - `edge_rate`: The probability of adding an edge.
    pub fn new(vertex_rate: f32, edge_rate: f32) -> Self {
        GraphMutator {
            vertex_rate,
            edge_rate,
            allow_recurrent: true,
        }
    }

    /// Set the `allow_recurrent` flag to allow or disallow recurrent nodes in the graph.
    ///
    /// If `allow` is true, recurrent nodes are allowed. If false, they are not.
    /// When a recurrent node is or cycle is created during mutation and `allow_recurrent` is false,
    /// the mutation will be discarded and the changes to the graph will be rolled back resulting in
    /// no changes to the graph.
    pub fn allow_recurrent(mut self, allow: bool) -> Self {
        self.allow_recurrent = allow;
        self
    }

    /// Get the type of node to add to the graph. This is used to determine if the node
    /// should be an edge or a vertex. First, a random boolean is generated. If true,
    /// we attempt to add an edge. If false, we attempt to add a vertex.
    fn mutate_type(&self) -> Option<NodeType> {
        if random_provider::bool(0.5) {
            if random_provider::bool(self.edge_rate) {
                Some(NodeType::Edge)
            } else {
                None
            }
        } else if random_provider::bool(self.vertex_rate) {
            Some(NodeType::Vertex)
        } else {
            None
        }
    }
}

impl<T> Mutate<GraphChromosome<T>> for GraphMutator
where
    T: Clone + PartialEq + Default,
{
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut GraphChromosome<T>, _: f32) -> AlterResult {
        // If the chromosome has a maximum number of nodes then just return 0.
        // If we have reached this point, this graph is simply optimizing the
        // node's values and not the structure.
        if let Some(max_nodes) = chromosome.max_nodes() {
            if chromosome.len() >= max_nodes {
                return AlterResult::empty();
            }
        }

        // Else, if we are below the maximum number of nodes,
        // attempt to mutate the graph by adding a new node of the determined type.
        if let Some(node_type) = self.mutate_type()
            && let Some(store) = chromosome.store()
        {
            let new_node = store.new_instance((chromosome.len(), node_type));
            let mut graph = Graph::new(chromosome.take_nodes());

            let result = graph.try_modify(|mut trans| {
                let needed_insertions = match new_node.arity() {
                    Arity::Exact(n) => n,
                    _ => 1,
                };

                let target_idx = trans.random_target_node().map(|n| n.index());
                let source_idx = (0..needed_insertions)
                    .filter_map(|_| trans.random_source_node().map(|n| n.index()))
                    .collect::<SmallVec<[_; 4]>>();

                let node_idx = trans.add_node(new_node);

                if let Some(trgt) = target_idx {
                    for src in source_idx {
                        let insertion_type = trans.get_insertion_steps(src, trgt, node_idx);

                        for step in insertion_type {
                            match step {
                                InsertStep::Connect(source, target) => trans.attach(source, target),
                                InsertStep::Detach(source, target) => trans.detach(source, target),
                                _ => {}
                            }
                        }
                    }
                }

                trans.commit_with(|graph: &Graph<T>| {
                    self.allow_recurrent || !graph.iter().any(|node| node.is_recurrent())
                })
            });

            chromosome.set_nodes(graph.take_nodes());

            return match result {
                TransactionResult::Invalid(_, _) => AlterResult::from(metric!(INVALID_MUTATION)),
                TransactionResult::Valid(steps) => AlterResult::from(steps.len()),
            };
        }

        AlterResult::empty()
    }
}
