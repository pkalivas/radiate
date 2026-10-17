use super::transaction::TransactionResult;
use super::{Graph, GraphChromosome};
use crate::graphs::node::InnovationId;
use crate::{Factory, NodeType};
use radiate_core::{AlterContext, Chromosome, Expr, RateSet, SmallStr, expr};
use radiate_core::{Mutate, random_provider};
use std::{collections::HashMap, fmt::Debug};

const SATURATED: SmallStr = SmallStr::from_static("mutator.graph.invalid.saturated");
const REJECTED: SmallStr = SmallStr::from_static("mutator.graph.invalid");
const ADD_VERTEX_RATE: SmallStr = SmallStr::from_static("mutator.graph.rate.vertex");
const ADD_EDGE_RATE: SmallStr = SmallStr::from_static("mutator.graph.rate.edge");

type InnovationKeyItem = Vec<InnovationId>;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct StructureChange {
    source_id: InnovationKeyItem,
    target_id: InnovationKeyItem,
}

#[derive(Debug, Clone)]
struct InnovationCache {
    version: usize,
    innovations: HashMap<StructureChange, InnovationId>,
}

impl InnovationCache {
    fn new() -> Self {
        InnovationCache {
            version: 0,
            innovations: HashMap::new(),
        }
    }

    fn bump(&mut self, next: usize) {
        if next > self.version {
            self.innovations.clear();
            self.version = next;
        }
    }

    fn get_innovation(
        &mut self,
        source_id: InnovationKeyItem,
        target_id: InnovationKeyItem,
    ) -> InnovationId {
        let change = StructureChange {
            source_id,
            target_id,
        };
        if let Some(id) = self.innovations.get(&change) {
            *id
        } else {
            let new_id = InnovationId::new();
            self.innovations.insert(change, new_id);
            new_id
        }
    }
}

/// A graph mutator that can be used to alter the graph structure within a [`GraphChromosome<T>`].
/// By adding new vertices and edges to the graph, it can be used to explore the search space of a graph.
///
/// # Arguments
/// - `vertex_rate`: The probability of adding a vertex.
/// - `edge_rate`: The probability of adding an edge.
/// - `allow_recurrent`: If true, recurrent nodes are allowed. If false, they are not. Default is true.
#[derive(Clone, Debug)]
pub struct GraphMutator {
    vertex_rate: Expr,
    edge_rate: Expr,
    allow_recurrent: bool,
    cache: InnovationCache,
}

impl GraphMutator {
    /// Create a new graph mutator with a set of mutations
    ///
    /// # Arguments
    /// - `vertex_rate`: The probability of adding a vertex.
    /// - `edge_rate`: The probability of adding an edge.
    pub fn new(vertex_rate: impl Into<Expr>, edge_rate: impl Into<Expr>) -> Self {
        GraphMutator {
            vertex_rate: vertex_rate.into(),
            edge_rate: edge_rate.into(),
            allow_recurrent: false,
            cache: InnovationCache::new(),
        }
    }

    /// Set the `allow_recurrent` flag to allow or disallow recurrent nodes in the graph.
    ///
    /// If `allow` is true, recurrent nodes/cycles can be added to the graph. If false, they are not.
    /// When a recurrent node is or cycle is created during mutation and `allow_recurrent` is false,
    /// the mutation will be discarded and the changes to the graph will be rolled back resulting in
    /// no changes to the graph.
    pub fn allow_recurrent(mut self) -> Self {
        self.allow_recurrent = true;
        self
    }

    /// Set the target size of the graph. If the graph is at or above this size
    /// the mutation rates will be reduced to prevent the graph from growing too large.
    pub fn target_size(mut self, size: usize) -> Self {
        self.vertex_rate = expr::genome_size_throttle(self.vertex_rate.clone(), size);
        self.edge_rate = expr::genome_size_throttle(self.edge_rate.clone(), size);
        self
    }

    /// Get the type of node to add to the graph. This is used to determine if the node
    /// should be an edge or a vertex. First, a random boolean is generated. If true,
    /// we attempt to add an edge. If false, we attempt to add a vertex.
    fn mutate_type(&self, ctx: &AlterContext) -> Option<NodeType> {
        random_provider::with_rng(|rand| {
            if rand.bool(0.5) {
                if rand.bool(ctx.internal_rate(0)) {
                    Some(NodeType::Edge)
                } else {
                    None
                }
            } else if rand.bool(ctx.internal_rate(1)) {
                Some(NodeType::Vertex)
            } else {
                None
            }
        })
    }
}

impl<T> Mutate<GraphChromosome<T>> for GraphMutator
where
    T: Clone + Default,
{
    fn rates(&self) -> RateSet {
        RateSet::new(1.0)
            .push(self.edge_rate.clone().alias(ADD_EDGE_RATE))
            .push(self.vertex_rate.clone().alias(ADD_VERTEX_RATE))
    }

    #[inline]
    fn mutate_chromosome(
        &mut self,
        chromosome: &mut GraphChromosome<T>,
        ctx: &mut AlterContext,
    ) -> usize {
        // If the chromosome has a maximum number of nodes then just return 0.
        // If we have reached this point, this graph is simply optimizing the
        // node's values and not the structure.
        if let Some(max_nodes) = chromosome.max_nodes()
            && chromosome.len() >= max_nodes
        {
            ctx.upsert(SATURATED, 1);
            return 0;
        }

        self.cache.bump(ctx.generation());

        // Else, if we are below the maximum number of nodes,
        // attempt to mutate the graph by adding a new node of the determined type.
        if let Some(node_type) = self.mutate_type(ctx)
            && let Some(store) = chromosome.store()
        {
            let Some(new_node) = store.new_instance((chromosome.len(), node_type)) else {
                return 0;
            };

            let mut graph = Graph::new(chromosome.take_nodes());

            let result = graph.try_modify(|mut trans| {
                let Some(node_idx) = trans.push_and_fill(new_node, self.allow_recurrent) else {
                    return trans.commit();
                };

                let incoming_innov = trans
                    .iter_node_incoming(node_idx)
                    .filter_map(|node| node.innovation())
                    .collect::<InnovationKeyItem>();
                let outgoing_innov = trans
                    .iter_node_outgoing(node_idx)
                    .filter_map(|node| node.innovation())
                    .collect::<InnovationKeyItem>();

                let innovation_id = self.cache.get_innovation(incoming_innov, outgoing_innov);

                trans.set_innovation(node_idx, Some(innovation_id));
                trans.commit_with(|graph: &Graph<T>| {
                    self.allow_recurrent || !graph.iter().any(|node| node.is_recurrent())
                })
            });

            chromosome.set_nodes(graph.take_nodes());

            return match result {
                TransactionResult::Invalid(_, _) => {
                    ctx.upsert(REJECTED, 1);
                    0
                }
                TransactionResult::Valid(_) => 1,
            };
        }

        0
    }
}
