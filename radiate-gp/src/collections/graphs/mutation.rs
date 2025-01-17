use radiate::{
    random_provider, timer::Timer, Alter, AlterAction, EngineCompoment, Metric, Mutate, Population,
    Valid,
};

use super::transaction::GraphTransaction;
use super::{Graph, GraphChromosome, GraphNode};
use crate::ops::Arity;
use crate::{CellStore, Factory, NodeCell, NodeType};

/// A node mutation used to alter the graph structure randomly
/// The mutation can be either an edge or a vertex, with a rate of mutation and a flag to
/// indicate if the node is recurrent. Note - at this point this only represents additions
/// to the `graph`.
pub enum NodeMutate {
    Edge(f32, bool),
    Vertex(f32, bool),
}

impl NodeMutate {
    pub fn node_type(&self) -> NodeType {
        match self {
            NodeMutate::Edge(_, _) => NodeType::Edge,
            NodeMutate::Vertex(_, _) => NodeType::Vertex,
        }
    }

    pub fn rate(&self) -> f32 {
        match self {
            NodeMutate::Edge(rate, _) => *rate,
            NodeMutate::Vertex(rate, _) => *rate,
        }
    }

    pub fn is_recurrent(&self) -> bool {
        match self {
            NodeMutate::Edge(_, rec) => *rec,
            NodeMutate::Vertex(_, rec) => *rec,
        }
    }
}

/// A graph mutator that can be used to alter the graph structure. This is used to add nodes
/// to the graph, and can be used to add either edges or vertices. The mutator is created with
/// a set of mutations that can be applied to the graph.
pub struct GraphMutator {
    mutations: Vec<NodeMutate>,
}

// updated GraphMutator implementation
impl GraphMutator {
    /// Create a new graph mutator with a set of mutations
    ///
    /// # Arguments
    /// - `mutations` - a vector of `NodeMutate` that represent the mutations that can be applied
    pub fn new(mutations: Vec<NodeMutate>) -> Self {
        Self { mutations }
    }

    /// Add a node to the graph using the transaction. This will attempt to add a node to the graph
    /// and if successful will commit the transaction. If the node cannot be added the transaction
    /// will be rolled back.
    pub fn add_node<C: NodeCell + Clone + Default + PartialEq>(
        &self,
        graph: &mut Graph<C>,
        node_type: &NodeType,
        factory: &CellStore<C>,
        recurrent: bool,
    ) -> bool {
        let mut transaction = GraphTransaction::new(graph);

        if !self.try_add_node(&mut transaction, node_type, factory, recurrent) {
            transaction.rollback();
            return false;
        }

        true
    }

    fn try_add_node<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        node_type: &NodeType,
        factory: &CellStore<C>,
        is_recurrent: bool,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        let source_node_index = transaction.as_ref().random_source_node().index();
        let target_node_index = transaction.as_ref().random_target_node().index();

        let source_node_type = transaction.as_ref()[source_node_index].node_type();

        if source_node_type == NodeType::Edge && node_type != &NodeType::Edge {
            if is_recurrent {
                self.try_backward_edge_insertion(
                    transaction,
                    source_node_index,
                    target_node_index,
                    node_type,
                    factory,
                )
            } else {
                self.try_edge_insertion(
                    transaction,
                    source_node_index,
                    target_node_index,
                    node_type,
                    factory,
                )
            }
        } else {
            self.try_normal_insertion(
                transaction,
                source_node_index,
                target_node_index,
                node_type,
                factory,
                is_recurrent,
            )
        }
    }

    fn try_edge_insertion<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        source_node: usize,
        target_node: usize,
        node_type: &NodeType,
        factory: &CellStore<C>,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        let new_source_edge_index = transaction.as_ref().len();
        let new_node_index = transaction.as_ref().len() + 1;
        let new_target_edge_index = transaction.as_ref().len() + 2;

        let source_node = transaction.as_ref()[source_node].index();
        let target_node = transaction.as_ref()[target_node].index();

        if transaction.as_ref()[target_node].is_locked() {
            let edge = factory.new_instance((
                new_source_edge_index,
                transaction.as_ref()[source_node].node_type(),
            ));
            let new_node = factory.new_instance((new_node_index, *node_type));

            let edge_index = transaction.add_node(edge);
            let node_index = transaction.add_node(new_node);

            transaction.attach(source_node, node_index);
            transaction.attach(node_index, edge_index);
            transaction.attach(edge_index, target_node);
            transaction.detach(source_node, target_node);
        } else {
            let new_source_edge = factory.new_instance((
                new_source_edge_index,
                transaction.as_ref()[source_node].node_type(),
            ));
            let new_node = factory.new_instance((new_node_index, *node_type));
            let new_target_edge = factory.new_instance((
                new_target_edge_index,
                transaction.as_ref()[target_node].node_type(),
            ));

            transaction.add_node(new_source_edge);
            transaction.add_node(new_node);
            transaction.add_node(new_target_edge);

            transaction.attach(source_node, new_source_edge_index);
            transaction.attach(new_source_edge_index, new_node_index);
            transaction.attach(new_node_index, new_target_edge_index);
            transaction.attach(new_target_edge_index, target_node);
        }

        self.complete_node_arity(transaction, new_node_index, factory, false)
    }

    fn try_backward_edge_insertion<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        source_idx: usize,
        target_idx: usize,
        node_type: &NodeType,
        factory: &CellStore<C>,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        let new_source_edge_index = transaction.as_ref().len();
        let new_node_index = transaction.as_ref().len() + 1;
        let new_target_edge_index = transaction.as_ref().len() + 2;
        let recurrent_edge_index = transaction.as_ref().len() + 3;

        // Get node info before mutations
        let source_node_type = transaction.as_ref()[source_idx].node_type();
        let source_is_recurrent = transaction.as_ref()[source_idx].is_recurrent();
        let source_incoming = transaction.as_ref()[source_idx].incoming().clone();
        let source_outgoing = transaction.as_ref()[source_idx].outgoing().clone();
        let target_is_locked = transaction.as_ref()[target_idx].is_locked();

        if source_node_type == NodeType::Edge && node_type != &NodeType::Edge {
            let incoming_idx = *source_incoming.iter().next().unwrap();
            let outgoing_idx = *source_outgoing.iter().next().unwrap();

            if target_is_locked {
                let new_node = factory.new_instance((new_node_index, *node_type));

                let new_source_edge =
                    factory.new_instance((new_source_edge_index, source_node_type));
                let new_target_edge =
                    factory.new_instance((new_target_edge_index, source_node_type));

                transaction.add_node(new_source_edge);
                transaction.add_node(new_node);
                transaction.add_node(new_target_edge);

                transaction.attach(incoming_idx, new_node_index);
                transaction.attach(new_node_index, new_source_edge_index);
                transaction.attach(new_source_edge_index, new_node_index);
                transaction.attach(new_node_index, new_target_edge_index);
                transaction.attach(new_target_edge_index, outgoing_idx);
                transaction.detach(incoming_idx, outgoing_idx);
            } else if !source_is_recurrent {
                let new_node = factory.new_instance((new_node_index, *node_type));

                let new_source_edge =
                    factory.new_instance((new_source_edge_index, source_node_type));
                let new_target_edge =
                    factory.new_instance((new_target_edge_index, source_node_type));
                let recurrent_edge = factory.new_instance((recurrent_edge_index, source_node_type));

                transaction.add_node(new_source_edge);
                transaction.add_node(new_node);
                transaction.add_node(new_target_edge);
                transaction.add_node(recurrent_edge);

                transaction.attach(incoming_idx, new_source_edge_index);
                transaction.attach(new_source_edge_index, new_node_index);
                transaction.attach(new_node_index, new_target_edge_index);
                transaction.attach(new_target_edge_index, outgoing_idx);
                transaction.attach(recurrent_edge_index, new_node_index);
                transaction.attach(new_node_index, recurrent_edge_index);
            } else {
                let new_node = factory.new_instance((new_node_index, *node_type));

                let new_source_edge =
                    factory.new_instance((new_source_edge_index, source_node_type));
                let new_target_edge =
                    factory.new_instance((new_target_edge_index, source_node_type));

                transaction.add_node(new_source_edge);
                transaction.add_node(new_node);
                transaction.add_node(new_target_edge);

                transaction.attach(incoming_idx, new_source_edge_index);
                transaction.attach(new_source_edge_index, new_node_index);
                transaction.attach(new_node_index, new_target_edge_index);
                transaction.attach(new_target_edge_index, outgoing_idx);
            }

            self.complete_node_arity(transaction, new_node_index, factory, true)
        } else {
            if !&transaction
                .as_ref()
                .can_connect(source_idx, target_idx, true)
            {
                return false;
            }

            let new_node = factory.new_instance((new_node_index, *node_type));

            transaction.add_node(new_node);

            transaction.attach(source_idx, new_node_index);
            transaction.attach(new_node_index, target_idx);
            transaction.detach(source_idx, target_idx);

            self.complete_node_arity(transaction, new_node_index, factory, true)
        }
    }

    fn try_normal_insertion<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        source_node: usize,
        target_node: usize,
        node_type: &NodeType,
        factory: &CellStore<C>,
        is_recurrent: bool,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        if !&transaction
            .as_ref()
            .can_connect(source_node, target_node, is_recurrent)
        {
            return false;
        }

        let new_node = factory.new_instance((transaction.as_ref().len(), *node_type));

        let node_index = transaction.add_node(new_node);
        transaction.attach(source_node, node_index);
        transaction.attach(node_index, target_node);
        transaction.detach(source_node, target_node);

        self.complete_node_arity(transaction, node_index, factory, is_recurrent)
    }

    fn complete_node_arity<C>(
        &self,
        transaction: &mut GraphTransaction<C>,
        node_index: usize,
        factory: &CellStore<C>,
        is_recurrent: bool,
    ) -> bool
    where
        C: Clone + Default + PartialEq + NodeCell,
    {
        let arity = transaction.as_ref()[node_index].value().arity();

        match arity {
            Arity::Any | Arity::Zero => {
                transaction.set_cycles();
                return transaction.as_ref().is_valid();
            }
            Arity::Exact(arity) => {
                for _ in 0..arity - 1 {
                    if random_provider::random::<f32>() < 0.05 {
                        let input_node =
                            factory.new_instance((transaction.as_ref().len(), NodeType::Input));
                        let input_index = transaction.add_node(input_node);
                        transaction.attach(input_index, node_index);
                    } else {
                        let other_source_node = transaction.as_ref().random_source_node();
                        if transaction.as_ref().can_connect(
                            other_source_node.index(),
                            node_index,
                            is_recurrent,
                        ) {
                            transaction.attach(other_source_node.index(), node_index);
                        }
                    }
                }
            }
        }

        transaction.set_cycles();
        transaction.as_ref().is_valid()
    }
}

impl EngineCompoment for GraphMutator {
    fn name(&self) -> &'static str {
        "GraphMutator"
    }
}

impl<C> Alter<GraphChromosome<C>> for GraphMutator
where
    C: Clone + PartialEq + Default + NodeCell,
{
    fn rate(&self) -> f32 {
        1.0
    }

    fn to_alter(self) -> AlterAction<GraphChromosome<C>> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<C> Mutate<GraphChromosome<C>> for GraphMutator
where
    C: Clone + PartialEq + Default + NodeCell,
{
    #[inline]
    fn mutate(
        &self,
        population: &mut Population<GraphChromosome<C>>,
        generation: i32,
    ) -> Vec<Metric> {
        let timer = Timer::new();
        let mut count = 0;
        for i in 0..population.len() {
            let phenotype = &mut population[i];
            let genotype = &mut phenotype.genotype();

            let chromosome_index = random_provider::random::<usize>() % genotype.len();

            let chromosome = &mut phenotype.genotype_mut()[chromosome_index];

            if self.mutate_chromosome(chromosome) > 0 {
                count += 1;
                phenotype.set_score(None);
                phenotype.generation = generation;
            }
        }

        let mut result = Metric::new_operations(self.name());
        result.add_value(count as f32);
        result.add_duration(timer.duration());

        vec![result]
    }

    fn mutate_chromosome(&self, chromosome: &mut GraphChromosome<C>) -> i32 {
        let mutation = random_provider::choose(&self.mutations);

        if random_provider::random::<f32>() > mutation.rate() {
            return 0;
        }

        if let Some(ref factory) = chromosome.store {
            let mut graph = Graph::new(chromosome.nodes.clone());
            let node_fact = factory.borrow();

            if self.add_node(
                &mut graph,
                &mutation.node_type(),
                &node_fact,
                mutation.is_recurrent(),
            ) {
                chromosome.nodes = graph.into_iter().collect::<Vec<GraphNode<C>>>();
                return 1;
            }
        }

        0
    }
}
