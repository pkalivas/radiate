use super::transaction::{InsertStep, TransactionResult};
use super::{Graph, GraphChromosome};
use crate::graphs::node::InnovationId;
use crate::node::Node;
use crate::{Arity, Factory, NodeType};
use radiate_core::{AlterContext, Chromosome, SmallStr};
use radiate_core::{AlterResult, Mutate, random_provider};
use std::collections::HashMap;

const SATURATED: SmallStr = SmallStr::from_static("mutate.graph.invalid.saturated");
const NO_INSTANCE: SmallStr = SmallStr::from_static("mutate.graph.invalid.no_instance");
const REJECTED: SmallStr = SmallStr::from_static("mutate.graph.invalid.rejected");

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct StructureChange {
    souce_id: Option<InnovationId>,
    target_id: Option<InnovationId>,
    node_type: NodeType,
}

#[derive(Debug, Clone)]
pub struct InnovationContext {
    version: usize,
    innovations: HashMap<StructureChange, InnovationId>,
}

impl InnovationContext {
    fn new() -> Self {
        InnovationContext {
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
        source_id: Option<InnovationId>,
        target_id: Option<InnovationId>,
        node_type: NodeType,
    ) -> InnovationId {
        let change = StructureChange {
            souce_id: source_id.clone(),
            target_id: target_id.clone(),
            node_type,
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
    vertex_rate: f32,
    edge_rate: f32,
    allow_recurrent: bool,
    innov_context: InnovationContext,
}

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
            innov_context: InnovationContext::new(),
        }
    }

    /// Set the `allow_recurrent` flag to allow or disallow recurrent nodes in the graph.
    ///
    /// If `allow` is true, recurrent nodes/cycles can be added to the graph. If false, they are not.
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
        random_provider::with_rng(|rand| {
            if rand.bool(0.5) {
                if rand.bool(self.edge_rate) {
                    Some(NodeType::Edge)
                } else {
                    None
                }
            } else if rand.bool(self.vertex_rate) {
                Some(NodeType::Vertex)
            } else {
                None
            }
        })
    }
}

impl<T> Mutate<GraphChromosome<T>> for GraphMutator
where
    T: Clone + PartialEq + Default,
{
    #[inline]
    fn mutate_chromosome(
        &mut self,
        chromosome: &mut GraphChromosome<T>,
        ctx: &mut AlterContext,
    ) -> AlterResult {
        // If the chromosome has a maximum number of nodes then just return 0.
        // If we have reached this point, this graph is simply optimizing the
        // node's values and not the structure.
        if let Some(max_nodes) = chromosome.max_nodes()
            && chromosome.len() >= max_nodes
        {
            ctx.metric(SATURATED, 1);
            return AlterResult::empty();
        }

        self.innov_context.bump(ctx.generation());

        // Else, if we are below the maximum number of nodes,
        // attempt to mutate the graph by adding a new node of the determined type.
        if let Some(node_type) = self.mutate_type()
            && let Some(store) = chromosome.store()
        {
            let Some(new_node) = store.new_instance((chromosome.len(), node_type)) else {
                ctx.metric(NO_INSTANCE, 1);
                return AlterResult::empty();
            };

            let mut graph = Graph::new(chromosome.take_nodes());

            let result = random_provider::with_rng(|rand| {
                graph.try_modify(|mut trans| {
                    let needed_insertions = match new_node.arity() {
                        Arity::Exact(n) => n,
                        _ => 1,
                    };

                    let target_idx = trans.random_target_node(rand).map(|n| n.index());
                    let source_idx = (0..needed_insertions)
                        .filter_map(|_| trans.random_source_node(rand).map(|n| n.index()))
                        .collect::<Vec<usize>>();

                    let node_idx = trans.push(new_node);

                    if let Some(trgt) = target_idx {
                        for src in source_idx {
                            let insertion_steps =
                                trans.get_insertion_steps(src, trgt, node_idx, rand);

                            for step in insertion_steps {
                                match step {
                                    InsertStep::Connect(source, target) => {
                                        trans.attach(source, target)
                                    }
                                    InsertStep::Detach(source, target) => {
                                        trans.detach(source, target)
                                    }
                                    InsertStep::NewStructure(
                                        source,
                                        new_node,
                                        target,
                                        node_type,
                                    ) => {
                                        let in_innov =
                                            trans.get(source).and_then(|n| n.innovation());
                                        let out_innov =
                                            trans.get(target).and_then(|n| n.innovation());

                                        let innov_id = self
                                            .innov_context
                                            .get_innovation(in_innov, out_innov, node_type);

                                        trans.set_innovation(new_node, Some(innov_id));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    trans.commit_with(|graph: &Graph<T>| {
                        self.allow_recurrent || !graph.iter().any(|node| node.is_recurrent())
                    })
                })
            });

            chromosome.set_nodes(graph.take_nodes());

            return match result {
                TransactionResult::Invalid(_, _) => {
                    ctx.metric(REJECTED, 1);
                    AlterResult::empty()
                }
                TransactionResult::Valid(steps) => AlterResult::from(steps.len()),
            };
        }

        AlterResult::empty()
    }
}
