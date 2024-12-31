use crate::collections::graphs::architect::GraphArchitect;
use crate::collections::{Builder, Graph, GraphNode, NodeFactory, NodeType};
use crate::ops::{Arity, Operation};
use crate::{ops, NodeCell};
use radiate::random_provider;
use std::collections::HashMap;

/// The `GraphBuilder` is a builder pattern that allows us to create a variety of different
/// graph architectures.
///
/// # Type Parameters
/// 'T': The type of the values that the graph will contain.
///
#[derive(Default)]
pub struct GraphBuilder<C: Clone + Default> {
    node_factory: Option<HashMap<NodeType, Vec<C>>>,
}

impl<C: Clone + Default> GraphBuilder<C> {
    pub fn new(node_factory: &NodeFactory<C>) -> Self {
        GraphBuilder {
            node_factory: Some(node_factory.node_values.clone()),
        }
    }

    pub fn from_factory(node_factory: HashMap<NodeType, Vec<C>>) -> Self {
        GraphBuilder {
            node_factory: Some(node_factory),
        }
    }
}

/// Configuration methods for the `GraphBuilder` that allow us to specify the different
/// types of nodes that the graph will contain.
impl<C: Clone + Default> GraphBuilder<C> {
    pub fn with_inputs(mut self, inputs: Vec<C>) -> Self {
        self.set_values(NodeType::Input, inputs);
        self
    }

    pub fn with_outputs(mut self, outputs: Vec<C>) -> Self {
        self.set_values(NodeType::Output, outputs);
        self
    }

    pub fn with_vertices(mut self, vertices: Vec<C>) -> Self {
        self.set_values(NodeType::Vertex, vertices);
        self
    }

    pub fn with_edges(mut self, edges: Vec<C>) -> Self {
        self.set_values(NodeType::Edge, edges);
        self
    }

    fn set_values(&mut self, node_type: NodeType, values: Vec<C>) {
        if let Some(ref mut factory) = self.node_factory {
            factory.insert(node_type, values);
        } else {
            let mut factory = HashMap::new();
            factory.insert(node_type, values);
            self.node_factory = Some(factory);
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
            .map(|i| {
                if let Some(values) = self.node_factory.as_ref().unwrap().get(&node_type) {
                    let value = values[i % values.len()].clone();
                    return GraphNode::new(i, node_type, value);
                }

                GraphNode::new(i, node_type, C::default())
            })
            .collect::<Vec<GraphNode<C>>>()
    }
}

impl<C: NodeCell + Clone + Default> GraphBuilder<C> {
    pub fn acyclic(&self, input_size: usize, output_size: usize) -> Graph<C> {
        GraphArchitect::new()
            .all_to_all(&self.input(input_size), &self.output(output_size))
            .build()
    }

    pub fn cyclic(&self, input_size: usize, output_size: usize) -> Graph<C> {
        let input = self.input(input_size);
        let aggregate = self.vertex(input_size);
        let link = self.vertex(input_size);
        let output = self.output(output_size);

        GraphArchitect::new()
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
///
impl GraphBuilder<Operation<f32>> {
    fn aggregates(&self, size: usize) -> Vec<GraphNode<Operation<f32>>> {
        let ops = self.operations_with_any_arity();
        (0..size)
            .map(|i| {
                let op = random_provider::choose(&ops).new_instance();
                GraphNode::new(i, NodeType::Vertex, op)
            })
            .collect::<Vec<GraphNode<Operation<f32>>>>()
    }

    fn operations_with_any_arity(&self) -> Vec<Operation<f32>> {
        if let Some(ref factory) = self.node_factory {
            if let Some(values) = factory.get(&NodeType::Vertex) {
                let vertecies_with_any = values
                    .iter()
                    .filter(|op| op.arity() == Arity::Any)
                    .cloned()
                    .collect::<Vec<Operation<f32>>>();

                if !vertecies_with_any.is_empty() {
                    return vertecies_with_any;
                }
            } else if let Some(values) = factory.get(&NodeType::Output) {
                let outputs_with_any = values
                    .iter()
                    .filter(|op| op.arity() == Arity::Any)
                    .cloned()
                    .collect::<Vec<Operation<f32>>>();

                if !outputs_with_any.is_empty() {
                    return outputs_with_any;
                }
            }
        }

        ops::get_activation_operations()
    }
}

impl GraphBuilder<Operation<f32>> {
    pub fn regression(input_size: usize) -> Self {
        let factory = NodeFactory::regression(input_size);
        GraphBuilder::from_factory(factory.node_values.clone())
    }

    pub fn weighted_acyclic(&self, input_size: usize, output_size: usize) -> Graph<Operation<f32>> {
        let input = self.input(input_size);
        let output = self.output(output_size);
        let weights = self.edge(input_size * output_size);

        GraphArchitect::new()
            .one_to_many(&input, &weights)
            .many_to_one(&weights, &output)
            .build()
    }

    pub fn weighted_cyclic(
        &self,
        input_size: usize,
        output_size: usize,
        memory_size: usize,
    ) -> Graph<Operation<f32>> {
        let input = self.input(input_size);
        let output = self.output(output_size);
        let weights = self.edge(input_size * memory_size);
        let aggregate = self.aggregates(memory_size);
        let aggregate_weights = self.edge(memory_size);

        GraphArchitect::new()
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
    ) -> Graph<Operation<f32>> {
        let input = self.input(input_size);
        let output = self.output(output_size);

        let query_weights = self.edge(input_size * num_heads);
        let key_weights = self.edge(input_size * num_heads);
        let value_weights = self.edge(input_size * num_heads);

        let attention_scores = self.aggregates(num_heads);
        let attention_aggreg = self.aggregates(num_heads);

        GraphArchitect::new()
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

    pub fn hopfield(&self, input_size: usize, output_size: usize) -> Graph<Operation<f32>> {
        let input = self.input(input_size);
        let output = self.output(output_size);
        let aggregates = self.vertex(input_size);
        let weights = self.edge(input_size * output_size);

        GraphArchitect::new()
            .one_to_many(&input, &aggregates)
            .one_to_many(&aggregates, &weights)
            .many_to_one(&weights, &aggregates)
            .many_to_one(&aggregates, &output)
            .build()
    }

    pub fn lstm(
        &self,
        input_size: usize,
        output_size: usize,
        memory_size: usize,
    ) -> Graph<Operation<f32>> {
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

        GraphArchitect::new()
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

    pub fn gru(
        &self,
        input_size: usize,
        output_size: usize,
        memory_size: usize,
    ) -> Graph<Operation<f32>> {
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

        GraphArchitect::new()
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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn graph_builder_simple_acyclic_f32() {
        // let factory = NodeFactory::new()
        //     .inputs((0..2).map(Operation::var).collect())
        //     .vertices(vec![Operation::linear()])
        //     .outputs(vec![Operation::linear()]);

        // let node = factory.generate((0, NodeType::Vertex));
        let builder = GraphBuilder::<Operation<f32>>::default()
            .with_inputs(vec![Operation::var(0), Operation::var(1)])
            .with_outputs(vec![Operation::linear()]);

        let graph = builder.acyclic(2, 1);

        assert_eq!(graph.len(), 3);
    }

    #[test]
    fn graph_builder_simple_cyclic_f32() {
        let builder = GraphBuilder::<Operation<f32>>::default()
            .with_inputs(vec![Operation::var(0), Operation::var(1)])
            .with_vertices(vec![Operation::linear()])
            .with_outputs(vec![Operation::linear()]);

        let graph = builder.cyclic(2, 1);

        println!("{:?}", graph);
    }
}
