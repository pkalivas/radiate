use super::transaction::{InsertStep, TransactionResult};
use super::{Graph, GraphChromosome, GraphNode};
use crate::node::Node;
use crate::{Arity, Factory, NodeType};
use radiate::Chromosome;
use radiate::{Alter, AlterAction, EngineCompoment, Mutate, random_provider};

/// A graph mutator that can be used to alter the graph structure. This is used to add nodes
/// to the graph, and can be used to add either edges or vertices. The mutator is created with
/// a set of mutations that can be applied to the graph.
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
    /// - `mutations` - a vector of `NodeMutate` that represent the mutations that can be applied
    pub fn new(vertex_rate: f32, edge_rate: f32) -> Self {
        GraphMutator {
            vertex_rate,
            edge_rate,
            allow_recurrent: true,
        }
    }

    pub fn allow_recurrent(mut self, allow: bool) -> Self {
        self.allow_recurrent = allow;
        self
    }

    fn mutate_type(&self) -> Option<NodeType> {
        if random_provider::bool(0.5) {
            if random_provider::random::<f32>() < self.edge_rate {
                Some(NodeType::Edge)
            } else {
                None
            }
        } else if random_provider::random::<f32>() < self.vertex_rate {
            Some(NodeType::Vertex)
        } else {
            None
        }
    }
}

impl EngineCompoment for GraphMutator {
    fn name(&self) -> &'static str {
        "GraphMutator"
    }
}

impl<T> Alter<GraphChromosome<T>> for GraphMutator
where
    T: Clone + PartialEq + Default,
{
    fn rate(&self) -> f32 {
        1.0
    }

    fn to_alter(self) -> AlterAction<GraphChromosome<T>> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<T> Mutate<GraphChromosome<T>> for GraphMutator
where
    T: Clone + PartialEq + Default,
{
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut GraphChromosome<T>) -> i32 {
        if let Some(node_type_to_add) = self.mutate_type() {
            if let Some(store) = chromosome.store() {
                let new_node = store.new_instance((chromosome.len(), node_type_to_add));

                let mut graph = Graph::new(chromosome.iter().cloned().collect());

                let result = graph.try_modify(|mut trans| {
                    let needed_insertions = match new_node.arity() {
                        Arity::Zero => 1,
                        Arity::Any => 1,
                        Arity::Exact(n) => n,
                    };

                    let target_idx = trans.random_target_node().map(|n| n.index());
                    let source_idx = (0..needed_insertions)
                        .filter_map(|_| trans.random_source_node().map(|n| n.index()))
                        .collect::<Vec<usize>>();

                    let node_idx = trans.add_node(new_node);

                    if let Some(trgt) = target_idx {
                        for src in source_idx {
                            let insertion_type = trans.get_insertion_steps(src, trgt, node_idx);

                            for step in insertion_type {
                                match step {
                                    InsertStep::Connect(source, target) => {
                                        trans.attach(source, target)
                                    }
                                    InsertStep::Detach(source, target) => {
                                        trans.detach(source, target)
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    trans.commit_with(Some(&|graph: &Graph<T>| {
                        if !self.allow_recurrent {
                            return graph.iter().all(|node| !node.is_recurrent());
                        }

                        true
                    }))
                });

                if let TransactionResult::Valid(_) = result {
                    chromosome.set_nodes(graph.into_iter().collect::<Vec<GraphNode<T>>>());
                    return 1;
                }
            }
        }

        0
    }
}
