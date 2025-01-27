use std::sync::{Arc, RwLock};

use crate::collections::{Builder, Graph, GraphNode, NodeType};

use crate::ops::{Arity, Op};
use crate::{ops, Factory};
use radiate::random_provider;

use super::aggregate::GraphAggregate;
use super::codex::GraphCodex;
use super::NodeStore;

/// The `GraphBuilder` is a builder pattern that allows us to create a variety of different
/// graph architectures.
///
/// # Type Parameters
/// 'C' - The type of the node cell that the graph will contain.
pub struct GraphBuilder<T> {
    store: Arc<RwLock<NodeStore<T>>>,
    node_cache: Option<Vec<GraphNode<T>>>,
}

impl<T> GraphBuilder<T> {
    pub fn new(store: NodeStore<T>) -> Self {
        GraphBuilder {
            store: Arc::new(RwLock::new(store)),
            node_cache: None,
        }
    }

    pub fn with_store(&mut self, store: NodeStore<T>) -> Self {
        GraphBuilder {
            store: Arc::new(RwLock::new(store)),
            node_cache: None,
        }
    }

    pub fn into_codex(self) -> GraphCodex<T> {
        GraphCodex::new(self.store, self.node_cache)
    }
}

/// Builder methods for creating different types of nodes in the graph.
/// These methods will create a collection of nodes of the specified type and size,
/// then layer we can use these nodes to build various graph architectures.
impl<T: Clone + Default> GraphBuilder<T> {
    pub fn set_vertecies(self, vertecies: Vec<Op<T>>) -> Self {
        self.store
            .write()
            .unwrap()
            .add_values(NodeType::Vertex, vertecies);
        self
    }

    pub fn set_edges(self, edges: Vec<Op<T>>) -> Self {
        self.store
            .write()
            .unwrap()
            .add_values(NodeType::Edge, edges);
        self
    }

    pub fn with_values(&self, node_type: NodeType, values: Vec<Op<T>>) {
        self.store.write().unwrap().add_values(node_type, values);
    }

    pub fn input(&self, size: usize) -> Vec<GraphNode<T>> {
        self.new_nodes(NodeType::Input, size)
    }

    pub fn output(&self, size: usize) -> Vec<GraphNode<T>> {
        self.new_nodes(NodeType::Output, size)
    }

    pub fn vertex(&self, size: usize) -> Vec<GraphNode<T>> {
        self.new_nodes(NodeType::Vertex, size)
    }

    pub fn edge(&self, size: usize) -> Vec<GraphNode<T>> {
        self.new_nodes(NodeType::Edge, size)
    }

    fn new_nodes(&self, node_type: NodeType, size: usize) -> Vec<GraphNode<T>> {
        (0..size)
            .map(|i| (*self.store.read().unwrap()).new_instance((i, node_type)))
            .collect::<Vec<GraphNode<T>>>()
    }
}

/// For `Graph<T>` where `T` is a `Float` type we can use the `GraphBuilder` to create
/// a variety of different graph architectures. Such as LSTM, GRU, Attention Units, etc
/// but for those we need to provide the `GraphBuilder` with a way to generate the nodes
/// that accept a variable number of inputs. This makes sure that the `GraphBuilder` can
/// generate those nodes when needed.
impl GraphBuilder<f32> {
    fn aggregates(&self, size: usize) -> Vec<GraphNode<f32>> {
        let ops = self.operations_with_any_arity();
        (0..size)
            .map(|i| {
                let op = random_provider::choose(&ops).new_instance(());
                GraphNode::new(i, NodeType::Vertex, op)
            })
            .collect::<Vec<GraphNode<f32>>>()
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

    pub fn acyclic(
        mut self,
        input_size: usize,
        output_size: usize,
        output: Op<f32>,
    ) -> GraphBuilder<f32> {
        self.with_values(NodeType::Input, (0..input_size).map(Op::var).collect());
        self.with_values(NodeType::Output, vec![output]);

        let graph = GraphAggregate::new()
            .all_to_all(&self.input(input_size), &self.output(output_size))
            .build();

        self.node_cache = Some(graph.into_iter().collect());
        self
    }

    pub fn cyclic(
        mut self,
        input_size: usize,
        output_size: usize,
        output: Op<f32>,
    ) -> GraphBuilder<f32> {
        self.with_values(NodeType::Input, (0..input_size).map(Op::var).collect());
        self.with_values(NodeType::Output, vec![output]);

        let input = self.input(input_size);
        let aggregate = self.aggregates(input_size);
        let link = self.aggregates(input_size);
        let output = self.output(output_size);

        let graph = GraphAggregate::new()
            .one_to_one(&input, &aggregate)
            .one_to_one_self(&aggregate, &link)
            .all_to_all(&aggregate, &output)
            .build();

        self.node_cache = Some(graph.into_iter().collect());
        self
    }

    pub fn weighted_acyclic(
        mut self,
        input_size: usize,
        output_size: usize,
        output: Op<f32>,
    ) -> GraphBuilder<f32> {
        self.with_values(NodeType::Input, (0..input_size).map(Op::var).collect());
        self.with_values(NodeType::Output, vec![output]);

        let input = self.input(input_size);
        let output = self.output(output_size);
        let weights = self.edge(input_size * output_size);

        let graph = GraphAggregate::new()
            .one_to_many(&input, &weights)
            .many_to_one(&weights, &output)
            .build();

        self.node_cache = Some(graph.into_iter().collect());
        self
    }

    pub fn weighted_cyclic(
        mut self,
        input_size: usize,
        output_size: usize,
        memory_size: usize,
        output: Op<f32>,
    ) -> GraphBuilder<f32> {
        self.with_values(NodeType::Input, (0..input_size).map(Op::var).collect());
        self.with_values(NodeType::Output, vec![output]);

        let input = self.input(input_size);
        let output = self.output(output_size);
        let weights = self.edge(input_size * memory_size);
        let aggregate = self.aggregates(memory_size);
        let aggregate_weights = self.edge(memory_size);

        let graph = GraphAggregate::new()
            .one_to_many(&input, &weights)
            .many_to_one(&weights, &aggregate)
            .one_to_one_self(&aggregate, &aggregate_weights)
            .all_to_all(&aggregate, &output)
            .build();

        self.node_cache = Some(graph.into_iter().collect());
        self
    }

    pub fn gru(
        mut self,
        input_size: usize,
        output_size: usize,
        memory_size: usize,
        output: Op<f32>,
    ) -> GraphBuilder<f32> {
        self.with_values(NodeType::Input, (0..input_size).map(Op::var).collect());
        self.with_values(NodeType::Output, vec![output]);

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

        let graph = GraphAggregate::new()
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
            .build();

        self.node_cache = Some(graph.into_iter().collect());
        self
    }

    pub fn lstm(
        mut self,
        input_size: usize,
        output_size: usize,
        output: Op<f32>,
    ) -> GraphBuilder<f32> {
        self.with_values(NodeType::Input, (0..input_size).map(Op::var).collect());
        self.with_values(NodeType::Output, vec![output]);

        let input = self.input(input_size);
        let output = self.output(output_size);

        let cell_state = self.aggregates(1);
        let hidden_state = self.aggregates(1);

        let forget_gate = self.aggregates(1);
        let input_gate = self.aggregates(1);
        let output_gate = self.aggregates(1);
        let candidate = self.aggregates(1);

        let input_to_forget_weights = self.edge(input_size);
        let input_to_input_weights = self.edge(input_size);
        let input_to_output_weights = self.edge(input_size);
        let input_to_candidate_weights = self.edge(input_size);

        let hidden_to_forget_weights = self.edge(1);
        let hidden_to_input_weights = self.edge(1);
        let hidden_to_output_weights = self.edge(1);
        let hidden_to_candidate_weights = self.edge(1);

        let final_weights = self.edge(output_size);

        let graph = GraphAggregate::new()
            .one_to_one(&input, &input_to_forget_weights)
            .one_to_one(&input, &input_to_input_weights)
            .one_to_one(&input, &input_to_output_weights)
            .one_to_one(&input, &input_to_candidate_weights)
            .one_to_one(&hidden_state, &hidden_to_forget_weights)
            .one_to_one(&hidden_state, &hidden_to_input_weights)
            .one_to_one(&hidden_state, &hidden_to_output_weights)
            .one_to_one(&hidden_state, &hidden_to_candidate_weights)
            .many_to_one(&input_to_forget_weights, &forget_gate)
            .many_to_one(&hidden_to_forget_weights, &forget_gate)
            .many_to_one(&input_to_input_weights, &input_gate)
            .many_to_one(&hidden_to_input_weights, &input_gate)
            .many_to_one(&input_to_output_weights, &output_gate)
            .many_to_one(&hidden_to_output_weights, &output_gate)
            .many_to_one(&input_to_candidate_weights, &candidate)
            .many_to_one(&hidden_to_candidate_weights, &candidate)
            .one_to_one(&forget_gate, &cell_state)
            .one_to_one(&input_gate, &candidate)
            .one_to_one(&candidate, &cell_state)
            .one_to_one(&cell_state, &hidden_state)
            .one_to_one(&output_gate, &hidden_state)
            .one_to_many(&hidden_state, &final_weights)
            .one_to_one(&final_weights, &output)
            .build();

        self.node_cache = Some(graph.into_iter().collect());
        self
    }
}

impl<T: Clone> Builder for GraphBuilder<T> {
    type Output = Graph<T>;

    fn build(&self) -> Self::Output {
        if let Some(nodes) = &self.node_cache {
            Graph::new(nodes.clone())
        } else {
            Graph::new(vec![])
        }
    }
}

impl Default for GraphBuilder<f32> {
    fn default() -> Self {
        let inputs = (0..1).map(Op::var).collect::<Vec<Op<f32>>>();
        let mut store = NodeStore::new();

        store.add_values(NodeType::Input, inputs);
        store.add_values(NodeType::Vertex, ops::get_all_operations());
        store.add_values(NodeType::Edge, vec![Op::weight(), Op::identity()]);
        store.add_values(NodeType::Output, vec![Op::linear()]);

        GraphBuilder {
            store: Arc::new(RwLock::new(store)),
            node_cache: None,
        }
    }
}
