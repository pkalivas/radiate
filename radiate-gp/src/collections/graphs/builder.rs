use std::sync::{Arc, RwLock};

use crate::collections::{Builder, Graph, GraphNode, NodeType};

use crate::ops::{Arity, Op};
use crate::{ops, CellStore, Factory, NodeCell};
use radiate::{random_provider, Chromosome, Codex, Genotype};

use super::aggregate::GraphAggregate;
use super::GraphChromosome;

/// The `GraphBuilder` is a builder pattern that allows us to create a variety of different
/// graph architectures.
///
/// # Type Parameters
/// 'C' - The type of the node cell that the graph will contain.
pub struct GraphBuilder<C: NodeCell> {
    store: Arc<RwLock<CellStore<C>>>,
    node_cache: Option<Vec<GraphNode<C>>>,
}

impl<C: NodeCell> GraphBuilder<C> {
    pub fn new(store: CellStore<C>) -> Self {
        GraphBuilder {
            store: Arc::new(RwLock::new(store)),
            node_cache: None,
        }
    }
}

/// Builder methods for creating different types of nodes in the graph.
/// These methods will create a collection of nodes of the specified type and size,
/// then layer we can use these nodes to build various graph architectures.
impl<C: NodeCell + Clone + Default> GraphBuilder<C> {
    pub fn input(&self, size: usize) -> Vec<GraphNode<C>> {
        self.new_nodes(NodeType::Input, size)
    }

    pub fn output(&self, size: usize) -> Vec<GraphNode<C>> {
        self.new_nodes(NodeType::Output, size)
    }

    pub fn vertex(&self, size: usize) -> Vec<GraphNode<C>> {
        self.new_nodes(NodeType::Vertex, size)
    }

    pub fn edge(&self, size: usize) -> Vec<GraphNode<C>> {
        self.new_nodes(NodeType::Edge, size)
    }

    fn new_nodes(&self, node_type: NodeType, size: usize) -> Vec<GraphNode<C>> {
        (0..size)
            .map(|i| (*self.store.read().unwrap()).new_instance((i, node_type)))
            .collect::<Vec<GraphNode<C>>>()
    }
}

impl<C: NodeCell + Clone + Default> GraphBuilder<C> {
    pub fn acyclic(mut self, input_size: usize, output_size: usize) -> GraphBuilder<C> {
        let graph = GraphAggregate::new()
            .all_to_all(&self.input(input_size), &self.output(output_size))
            .build();

        self.node_cache = Some(graph.into_iter().collect());
        self
    }

    pub fn cyclic(&self, input_size: usize, output_size: usize) -> Graph<C> {
        let input = self.input(input_size);
        let aggregate = self.vertex(input_size);
        let link = self.vertex(input_size);
        let output = self.output(output_size);

        GraphAggregate::new()
            .one_to_one(&input, &aggregate)
            .one_to_one_self(&aggregate, &link)
            .all_to_all(&aggregate, &output)
            .build()
    }
}

/// For `Graph<T>` where `T` is a `Float` type we can use the `GraphBuilder` to create
/// a variety of different graph architectures. Such as LSTM, GRU, Attention Units, etc
/// but for those we need to provide the `GraphBuilder` with a way to generate the nodes
/// that accept a variable number of inputs. This makes sure that the `GraphBuilder` can
/// generate those nodes when needed.
impl GraphBuilder<Op<f32>> {
    fn aggregates(&self, size: usize) -> Vec<GraphNode<Op<f32>>> {
        let ops = self.operations_with_any_arity();
        (0..size)
            .map(|i| {
                let op = random_provider::choose(&ops).new_instance();
                GraphNode::new(i, NodeType::Vertex, op)
            })
            .collect::<Vec<GraphNode<Op<f32>>>>()
    }

    fn operations_with_any_arity(&self) -> Vec<Op<f32>> {
        if let Some(values) = self.store.read().unwrap().get_values(NodeType::Vertex) {
            let vertecies_with_any = values
                .iter()
                .filter(|op| op.arity() == Arity::Any)
                .cloned()
                .collect::<Vec<Op<f32>>>();

            if !vertecies_with_any.is_empty() {
                return vertecies_with_any;
            }
        } else if let Some(values) = self.store.read().unwrap().get_values(NodeType::Output) {
            let outputs_with_any = values
                .iter()
                .filter(|op| op.arity() == Arity::Any)
                .cloned()
                .collect::<Vec<Op<f32>>>();

            if !outputs_with_any.is_empty() {
                return outputs_with_any;
            }
        }

        ops::get_activation_operations()
    }
}

impl GraphBuilder<Op<f32>> {
    pub fn regression(input_size: usize) -> Self {
        let store = CellStore::<Op<f32>>::regressor(input_size);
        GraphBuilder::new(store)
    }

    pub fn weighted_acyclic(&self, input_size: usize, output_size: usize) -> Graph<Op<f32>> {
        let input = self.input(input_size);
        let output = self.output(output_size);
        let weights = self.edge(input_size * output_size);

        GraphAggregate::new()
            .one_to_many(&input, &weights)
            .many_to_one(&weights, &output)
            .build()
    }

    pub fn weighted_cyclic(
        &self,
        input_size: usize,
        output_size: usize,
        memory_size: usize,
    ) -> Graph<Op<f32>> {
        let input = self.input(input_size);
        let output = self.output(output_size);
        let weights = self.edge(input_size * memory_size);
        let aggregate = self.aggregates(memory_size);
        let aggregate_weights = self.edge(memory_size);

        GraphAggregate::new()
            .one_to_many(&input, &weights)
            .many_to_one(&weights, &aggregate)
            .one_to_one_self(&aggregate, &aggregate_weights)
            .all_to_all(&aggregate, &output)
            .build()
    }

    pub fn attention_unit(
        &self,
        input_size: usize,
        output_size: usize,
        num_heads: usize,
    ) -> Graph<Op<f32>> {
        let input = self.input(input_size);
        let output = self.output(output_size);

        let query_weights = self.edge(input_size * num_heads);
        let key_weights = self.edge(input_size * num_heads);
        let value_weights = self.edge(input_size * num_heads);

        let attention_scores = self.aggregates(num_heads);
        let attention_aggreg = self.aggregates(num_heads);

        GraphAggregate::new()
            .one_to_many(&input, &query_weights)
            .one_to_many(&input, &key_weights)
            .one_to_many(&input, &value_weights)
            .many_to_one(&query_weights, &attention_scores)
            .many_to_one(&key_weights, &attention_scores)
            .one_to_many(&attention_scores, &attention_aggreg)
            .many_to_one(&value_weights, &attention_aggreg)
            .many_to_one(&attention_aggreg, &output)
            .build()
    }

    pub fn lstm(
        &self,
        input_size: usize,
        output_size: usize,
        memory_size: usize,
    ) -> Graph<Op<f32>> {
        let input = self.input(input_size);
        let output = self.output(output_size);

        let input_to_forget_weights = self.edge(input_size * memory_size);
        let hidden_to_forget_weights = self.edge(memory_size * memory_size);

        let input_to_input_weights = self.edge(input_size * memory_size);
        let hidden_to_input_weights = self.edge(memory_size * memory_size);

        let input_to_candidate_weights = self.edge(input_size * memory_size);
        let hidden_to_candidate_weights = self.edge(memory_size * memory_size);

        let input_to_output_weights = self.edge(input_size * memory_size);
        let hidden_to_output_weights = self.edge(memory_size * memory_size);

        let output_weights = self.edge(memory_size * output_size);

        let forget_gate = self.aggregates(memory_size);
        let input_gate = self.aggregates(memory_size);
        let candidate_gate = self.aggregates(memory_size);
        let output_gate = self.aggregates(memory_size);

        let input_candidate_mul_gate = self.aggregates(memory_size);
        let forget_memory_mul_gate = self.aggregates(memory_size);
        let memory_candidate_gate = self.aggregates(memory_size);
        let output_tahn_mul_gate = self.aggregates(memory_size);
        let tanh_gate = self.aggregates(memory_size);

        GraphAggregate::new()
            .one_to_many(&input, &input_to_forget_weights)
            .one_to_many(&input, &input_to_input_weights)
            .one_to_many(&input, &input_to_candidate_weights)
            .one_to_many(&input, &input_to_output_weights)
            .one_to_many(&output_tahn_mul_gate, &hidden_to_forget_weights)
            .one_to_many(&output_tahn_mul_gate, &hidden_to_input_weights)
            .one_to_many(&output_tahn_mul_gate, &hidden_to_candidate_weights)
            .one_to_many(&output_tahn_mul_gate, &hidden_to_output_weights)
            .many_to_one(&input_to_forget_weights, &forget_gate)
            .many_to_one(&hidden_to_forget_weights, &forget_gate)
            .many_to_one(&input_to_input_weights, &input_gate)
            .many_to_one(&hidden_to_input_weights, &input_gate)
            .many_to_one(&input_to_candidate_weights, &candidate_gate)
            .many_to_one(&hidden_to_candidate_weights, &candidate_gate)
            .many_to_one(&input_to_output_weights, &output_gate)
            .many_to_one(&hidden_to_output_weights, &output_gate)
            .one_to_one(&forget_gate, &forget_memory_mul_gate)
            .one_to_one(&memory_candidate_gate, &forget_memory_mul_gate)
            .one_to_one(&input_gate, &input_candidate_mul_gate)
            .one_to_one(&candidate_gate, &input_candidate_mul_gate)
            .one_to_one(&forget_memory_mul_gate, &memory_candidate_gate)
            .one_to_one(&input_candidate_mul_gate, &memory_candidate_gate)
            .one_to_one(&memory_candidate_gate, &tanh_gate)
            .one_to_one(&tanh_gate, &output_tahn_mul_gate)
            .one_to_one(&output_gate, &output_tahn_mul_gate)
            .one_to_many(&output_tahn_mul_gate, &output_weights)
            .many_to_one(&output_weights, &output)
            .build()
    }

    pub fn gru(&self, input_size: usize, output_size: usize, memory_size: usize) -> Graph<Op<f32>> {
        let input = self.input(input_size);
        let output = self.output(output_size);

        let output_weights = self.edge(memory_size * output_size);

        let reset_gate = self.aggregates(memory_size);
        let update_gate = self.aggregates(memory_size);
        let candidate_gate = self.aggregates(memory_size);

        let input_to_reset_weights = self.edge(input_size * memory_size);
        let input_to_update_weights = self.edge(input_size * memory_size);
        let input_to_candidate_weights = self.edge(input_size * memory_size);

        let hidden_to_reset_weights = self.edge(memory_size * memory_size);
        let hidden_to_update_weights = self.edge(memory_size * memory_size);
        let hidden_to_candidate_weights = self.edge(memory_size * memory_size);

        let hidden_reset_gate = self.aggregates(memory_size);
        let update_candidate_mul_gate = self.aggregates(memory_size);
        let invert_update_gate = self.aggregates(memory_size);
        let hidden_invert_mul_gate = self.aggregates(memory_size);
        let candidate_hidden_add_gate = self.aggregates(memory_size);

        GraphAggregate::new()
            .one_to_many(&input, &input_to_reset_weights)
            .one_to_many(&input, &input_to_update_weights)
            .one_to_many(&input, &input_to_candidate_weights)
            .one_to_many(&candidate_hidden_add_gate, &hidden_to_reset_weights)
            .one_to_many(&candidate_hidden_add_gate, &hidden_to_update_weights)
            .many_to_one(&input_to_reset_weights, &reset_gate)
            .many_to_one(&hidden_to_reset_weights, &reset_gate)
            .many_to_one(&input_to_update_weights, &update_gate)
            .many_to_one(&hidden_to_update_weights, &update_gate)
            .one_to_one(&reset_gate, &hidden_reset_gate)
            .one_to_one(&candidate_hidden_add_gate, &hidden_reset_gate)
            .one_to_many(&hidden_reset_gate, &hidden_to_candidate_weights)
            .many_to_one(&input_to_candidate_weights, &candidate_gate)
            .many_to_one(&hidden_to_candidate_weights, &candidate_gate)
            .one_to_one(&update_gate, &update_candidate_mul_gate)
            .one_to_one(&candidate_gate, &update_candidate_mul_gate)
            .one_to_one(&update_gate, &invert_update_gate)
            .one_to_one(&candidate_hidden_add_gate, &hidden_invert_mul_gate)
            .one_to_one(&invert_update_gate, &hidden_invert_mul_gate)
            .one_to_one(&hidden_invert_mul_gate, &candidate_hidden_add_gate)
            .one_to_one(&update_candidate_mul_gate, &candidate_hidden_add_gate)
            .one_to_many(&candidate_hidden_add_gate, &output_weights)
            .many_to_one(&output_weights, &output)
            .build()
    }
}

impl<C: NodeCell + Clone> Builder for GraphBuilder<C> {
    type Output = Graph<C>;

    fn build(&self) -> Self::Output {
        if let Some(nodes) = &self.node_cache {
            Graph::new(nodes.clone())
        } else {
            Graph::new(vec![])
        }
    }
}

impl<C> Codex<GraphChromosome<C>, Graph<C>> for GraphBuilder<C>
where
    C: NodeCell + Clone + PartialEq + Default,
{
    fn encode(&self) -> Genotype<GraphChromosome<C>> {
        let store = self.store.clone();
        let nodes = self.node_cache.clone().unwrap_or_default();

        Genotype::from_chromosomes(vec![GraphChromosome {
            nodes,
            store: Some(store),
        }])
    }

    fn decode(&self, genotype: &Genotype<GraphChromosome<C>>) -> Graph<C> {
        let nodes = genotype
            .iter()
            .next()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<GraphNode<C>>>();

        Graph::new(nodes)
    }
}

impl<C: NodeCell> Default for GraphBuilder<C> {
    fn default() -> Self {
        GraphBuilder {
            store: Arc::new(RwLock::new(CellStore::default())),
            node_cache: None,
        }
    }
}
