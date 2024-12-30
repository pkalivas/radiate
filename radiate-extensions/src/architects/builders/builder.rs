use std::collections::HashMap;

use crate::architect::GraphArchitect;
use crate::ops::operation::Operation;
use crate::{Graph, GraphNode, NodeFactory, NodeType, Tree, TreeNode};

use crate::ops::{operation, Arity};
use num_traits::{Float, NumCast};
use radiate::random_provider;
use rand::distributions::uniform::SampleUniform;
use rand::distributions::Standard;
use rand::prelude::Distribution;

use super::{Builder, Factory};

pub struct TreeBuilder<T> {
    gates: Vec<Operation<T>>,
    leafs: Vec<Operation<T>>,
    depth: usize,
}

impl<T> TreeBuilder<T>
where
    T: Clone,
{
    pub fn new(depth: usize) -> Self {
        TreeBuilder {
            gates: Vec::new(),
            leafs: Vec::new(),
            depth,
        }
    }

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    pub fn with_gates(mut self, gates: Vec<Operation<T>>) -> Self {
        self.gates = gates;
        self
    }

    pub fn with_leafs(mut self, leafs: Vec<Operation<T>>) -> Self {
        self.leafs = leafs;
        self
    }

    pub fn build(&self) -> Tree<T>
    where
        T: Default,
    {
        let root = self.grow_tree(self.depth);
        Tree::new(root)
    }

    fn grow_tree(&self, depth: usize) -> TreeNode<T>
    where
        T: Default,
    {
        if depth == 0 {
            let leaf = if self.leafs.is_empty() {
                Operation::default()
            } else {
                random_provider::choose(&self.leafs).new_instance()
            };

            return TreeNode::new(leaf);
        }

        let gate = if self.gates.is_empty() {
            Operation::default()
        } else {
            random_provider::choose(&self.gates).new_instance()
        };

        let mut parent = TreeNode::new(gate);
        for _ in 0..*parent.value.arity() {
            let temp = self.grow_tree(depth - 1);
            parent.add_child(temp);
        }

        parent
    }
}

/// The `GraphBuilder` is a builder pattern that allows us to create a variety of different
/// graph architectures.
///
/// # Type Parameters
/// 'T': The type of the values that the graph will contain.
///
#[derive(Default)]
pub struct GraphBuilder<T: Clone + Default> {
    node_factory: HashMap<NodeType, Vec<Operation<T>>>,
    layers: Vec<Graph<T>>,
}

impl<T: Clone + Default> GraphBuilder<T> {
    pub fn new(node_factory: &NodeFactory<T>) -> Self {
        GraphBuilder {
            node_factory: node_factory.node_values.clone(),
            layers: Vec::new(),
        }
    }

    pub fn from_factory(node_factory: HashMap<NodeType, Vec<Operation<T>>>) -> Self {
        GraphBuilder {
            node_factory,
            layers: Vec::new(),
        }
    }
}

/// Configuration methods for the `GraphBuilder` that allow us to specify the different
/// types of nodes that the graph will contain.
impl<T: Clone + Default> GraphBuilder<T> {
    pub fn with_inputs(mut self, inputs: Vec<Operation<T>>) -> Self {
        self.set_values(NodeType::Input, inputs);
        self
    }

    pub fn with_outputs(mut self, outputs: Vec<Operation<T>>) -> Self {
        self.set_values(NodeType::Output, outputs);
        self
    }

    pub fn with_vertices(mut self, vertices: Vec<Operation<T>>) -> Self {
        self.set_values(NodeType::Vertex, vertices);
        self
    }

    pub fn with_edges(mut self, edges: Vec<Operation<T>>) -> Self {
        self.set_values(NodeType::Edge, edges);
        self
    }

    fn set_values(&mut self, node_type: NodeType, values: Vec<Operation<T>>) {
        self.node_factory.insert(node_type, values);
    }
}

/// Builder methods for creating different types of nodes in the graph.
/// These methods will create a collection of nodes of the specified type and size,
/// then layer we can use these nodes to build various graph architectures.
impl<T: Clone + Default> GraphBuilder<T> {
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
            .map(|i| self.node_factory.new_instance((i, node_type)))
            .collect::<Vec<GraphNode<T>>>()
    }
}

impl<T> GraphBuilder<T>
where
    T: Clone + Default,
{
    pub fn acyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
        GraphArchitect::new()
            .all_to_all(&self.input(input_size), &self.output(output_size))
            .build()
    }

    pub fn cyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
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
impl<T> GraphBuilder<T>
where
    T: Clone + Default + Float,
{
    fn aggregates(&self, size: usize) -> Vec<GraphNode<T>> {
        let ops = self.operations_with_any_arity();
        (0..size)
            .map(|i| {
                let op = random_provider::choose(&ops).new_instance();
                GraphNode::new(i, NodeType::Vertex, op)
            })
            .collect::<Vec<GraphNode<T>>>()
    }

    fn operations_with_any_arity(&self) -> Vec<Operation<T>> {
        if let Some(values) = self.node_factory.get(&NodeType::Vertex) {
            let vertecies_with_any = values
                .iter()
                .filter(|op| op.arity() == Arity::Any)
                .cloned()
                .collect::<Vec<Operation<T>>>();

            if !vertecies_with_any.is_empty() {
                return vertecies_with_any;
            }
        } else if let Some(values) = self.node_factory.get(&NodeType::Output) {
            let outputs_with_any = values
                .iter()
                .filter(|op| op.arity() == Arity::Any)
                .cloned()
                .collect::<Vec<Operation<T>>>();

            if !outputs_with_any.is_empty() {
                return outputs_with_any;
            }
        }

        vec![
            operation::sum(),
            operation::prod(),
            operation::min(),
            operation::max(),
        ]
    }
}

impl<T> GraphBuilder<T>
where
    Standard: Distribution<T>,
    T: PartialOrd + NumCast + SampleUniform + Clone + Default + Float,
{
    pub fn regression(input_size: usize) -> Self {
        let factory = GraphBuilder::regression_factory(input_size);
        GraphBuilder::from_factory(factory)
    }

    pub fn weighted_acyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
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
    ) -> Graph<T> {
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
    ) -> Graph<T> {
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

    pub fn hopfield(&self, input_size: usize, output_size: usize) -> Graph<T> {
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

    pub fn lstm(&self, input_size: usize, output_size: usize, memory_size: usize) -> Graph<T> {
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

    pub fn gru(&self, input_size: usize, output_size: usize, memory_size: usize) -> Graph<T> {
        let input = self.input(input_size);
        let output = self.output(output_size);

        let output_weights = self.edge(memory_size * output_size);

        let reset_gate = self.vertex(memory_size);
        let update_gate = self.vertex(memory_size);
        let candidate_gate = self.vertex(memory_size);

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

    fn regression_factory(input_size: usize) -> HashMap<NodeType, Vec<Operation<T>>> {
        let inputs = (0..input_size)
            .map(operation::var)
            .collect::<Vec<Operation<T>>>();

        let vertices = vec![
            operation::add(),
            operation::sub(),
            operation::mul(),
            operation::div(),
            operation::pow(),
            operation::sqrt(),
            operation::exp(),
            operation::abs(),
            operation::log(),
            operation::sin(),
            operation::cos(),
            operation::tan(),
            operation::ceil(),
            operation::floor(),
            operation::gt(),
            operation::lt(),
            operation::sigmoid(),
            operation::tanh(),
            operation::relu(),
            operation::linear(),
            operation::max(),
            operation::min(),
            operation::mish(),
            operation::leaky_relu(),
            operation::softplus(),
            operation::sum(),
            operation::prod(),
        ];

        let edgess = vec![operation::weight(), operation::identity()];
        let outputs = vec![operation::linear()];

        let mut node_factory = HashMap::new();

        node_factory.insert(NodeType::Input, inputs.clone());
        node_factory.insert(NodeType::Vertex, vertices.clone());
        node_factory.insert(NodeType::Edge, edgess.clone());
        node_factory.insert(NodeType::Output, outputs.clone());

        node_factory
    }
}

impl<T> Builder for GraphBuilder<T>
where
    T: Clone + Default,
{
    type Output = Graph<T>;

    fn build(&self) -> Self::Output {
        let mut layers = Vec::new();
        for layer in &self.layers {
            layers.push(layer);
        }

        GraphArchitect::new().layer(layers).build()
    }
}

#[cfg(test)]
mod test {
    use crate::ops::operation;

    use super::*;

    #[test]
    fn graph_builder_simple_acyclic_f32() {
        // let factory = NodeFactory::new()
        //     .inputs((0..2).map(operation::var).collect())
        //     .vertices(vec![operation::linear()])
        //     .outputs(vec![operation::linear()]);

        // // let node = factory.generate((0, NodeType::Vertex));
        // let builder = GraphBuilder::<f32>::default()
        //     .with_inputs(vec![operation::var(0), operation::var(1)])
        //     .with_outputs(vec![operation::linear()]);

        // let graph = builder.acyclic(2, 1);

        // assert_eq!(graph.len(), 3);
    }

    #[test]
    fn graph_builder_simple_cyclic_f32() {
        let builder = GraphBuilder::<f32>::default()
            .with_inputs(vec![operation::var(0), operation::var(1)])
            .with_outputs(vec![operation::linear()]);

        let graph = builder.cyclic(2, 1);

        println!("{:?}", graph);
    }
}

// #[derive(Default)]
// pub struct GraphBuilder<T: Clone + Default> {
//     node_factory: NodeFactory<T>,
// }

// impl<T: Clone + Default> GraphBuilder<T> {
//     pub fn new(node_factory: &NodeFactory<T>) -> Self {
//         GraphBuilder {
//             node_factory: node_factory.clone(),
//         }
//     }

//     pub fn build<F>(&self, build_fn: F) -> Graph<T>
//     where
//         F: FnOnce(&GraphBuilder<T>, GraphArchitect<T>) -> Graph<T>,
//     {
//         build_fn(self, GraphArchitect::new(&self.node_factory))
//     }

//     pub fn input(&self, size: usize) -> Graph<T> {
//         self.new_collection(NodeType::Input, size)
//     }

//     pub fn output(&self, size: usize) -> Graph<T> {
//         self.new_collection(NodeType::Output, size)
//     }

//     pub fn vertex(&self, size: usize) -> Graph<T> {
//         self.new_collection(NodeType::Vertex, size)
//     }

//     pub fn edge(&self, size: usize) -> Graph<T> {
//         self.new_collection(NodeType::Edge, size)
//     }

//     pub fn new_collection(&self, node_type: NodeType, size: usize) -> Graph<T> {
//         let nodes = self.new_nodes(node_type, size);
//         Graph::new(nodes)
//     }

//     pub fn new_nodes(&self, node_type: NodeType, size: usize) -> Vec<GraphNode<T>> {
//         (0..size)
//             .map(|i| self.node_factory.new_node(i, node_type))
//             .collect::<Vec<GraphNode<T>>>()
//     }

//     pub fn with_inputs(mut self, inputs: Vec<Operation<T>>) -> Self {
//         self.set_values(NodeType::Input, inputs);
//         self
//     }

//     pub fn with_outputs(mut self, outputs: Vec<Operation<T>>) -> Self {
//         self.set_values(NodeType::Output, outputs);
//         self
//     }

//     pub fn with_vertices(mut self, vertices: Vec<Operation<T>>) -> Self {
//         self.set_values(NodeType::Vertex, vertices);
//         self
//     }

//     pub fn with_edges(mut self, edges: Vec<Operation<T>>) -> Self {
//         self.set_values(NodeType::Edge, edges);
//         self
//     }

//     fn set_values(&mut self, node_type: NodeType, values: Vec<Operation<T>>) {
//         self.node_factory.add_node_values(node_type, values);
//     }
// }

// impl<T> GraphBuilder<T>
// where
//     T: Clone + PartialEq + Default,
// {
//     pub fn acyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
//         GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
//             builder
//                 .all_to_all(&arc.input(input_size), &arc.output(output_size))
//                 .build()
//         })
//     }

//     pub fn cyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
//         GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let aggregate = arc.vertex(input_size);
//             let link = arc.vertex(input_size);
//             let output = arc.output(output_size);

//             builder
//                 .one_to_one(&input, &aggregate)
//                 .one_to_one_self(&aggregate, &link)
//                 .all_to_all(&aggregate, &output)
//                 .build()
//         })
//     }

//     pub fn weighted_acyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
//         GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let output = arc.output(output_size);
//             let weights = arc.edge(input_size * output_size);

//             builder
//                 .one_to_many(&input, &weights)
//                 .many_to_one(&weights, &output)
//                 .build()
//         })
//     }

//     pub fn weighted_cyclic(
//         &self,
//         input_size: usize,
//         output_size: usize,
//         memory_size: usize,
//     ) -> Graph<T> {
//         GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let output = arc.output(output_size);
//             let weights = arc.edge(input_size * memory_size);
//             let aggregate = arc.vertex(memory_size);
//             let aggregate_weights = arc.edge(memory_size);

//             builder
//                 .one_to_many(&input, &weights)
//                 .many_to_one(&weights, &aggregate)
//                 .one_to_one_self(&aggregate, &aggregate_weights)
//                 .all_to_all(&aggregate, &output)
//                 .build()
//         })
//     }

//     pub fn attention_unit(
//         &self,
//         input_size: usize,
//         output_size: usize,
//         num_heads: usize,
//     ) -> Graph<T> {
//         GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let output = arc.output(output_size);

//             let query_weights = arc.edge(input_size * num_heads);
//             let key_weights = arc.edge(input_size * num_heads);
//             let value_weights = arc.edge(input_size * num_heads);

//             let attention_scores = arc.new_collection(NodeType::Vertex, num_heads);
//             let attention_aggreg = arc.new_collection(NodeType::Vertex, num_heads);

//             builder
//                 .one_to_many(&input, &query_weights)
//                 .one_to_many(&input, &key_weights)
//                 .one_to_many(&input, &value_weights)
//                 .many_to_one(&query_weights, &attention_scores)
//                 .many_to_one(&key_weights, &attention_scores)
//                 .one_to_many(&attention_scores, &attention_aggreg)
//                 .many_to_one(&value_weights, &attention_aggreg)
//                 .many_to_one(&attention_aggreg, &output)
//                 .build()
//         })
//     }

//     pub fn hopfield(&self, input_size: usize, output_size: usize) -> Graph<T> {
//         GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let output = arc.output(output_size);
//             let aggregates = arc.vertex(input_size);
//             let weights = arc.edge(input_size * output_size);

//             builder
//                 .one_to_many(&input, &aggregates)
//                 .one_to_many(&aggregates, &weights)
//                 .many_to_one(&weights, &aggregates)
//                 .many_to_one(&aggregates, &output)
//                 .build()
//         })
//     }

//     pub fn lstm(&self, input_size: usize, output_size: usize, memory_size: usize) -> Graph<T> {
//         GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let output = arc.output(output_size);

//             let input_to_forget_weights = arc.edge(input_size * memory_size);
//             let hidden_to_forget_weights = arc.edge(memory_size * memory_size);

//             let input_to_input_weights = arc.edge(input_size * memory_size);
//             let hidden_to_input_weights = arc.edge(memory_size * memory_size);

//             let input_to_candidate_weights = arc.edge(input_size * memory_size);
//             let hidden_to_candidate_weights = arc.edge(memory_size * memory_size);

//             let input_to_output_weights = arc.edge(input_size * memory_size);
//             let hidden_to_output_weights = arc.edge(memory_size * memory_size);

//             let output_weights = arc.edge(memory_size * output_size);

//             let forget_gate = arc.vertex(memory_size);
//             let input_gate = arc.vertex(memory_size);
//             let candidate_gate = arc.vertex(memory_size);
//             let output_gate = arc.vertex(memory_size);

//             let input_candidate_mul_gate = arc.new_collection(NodeType::Vertex, memory_size);
//             let forget_memory_mul_gate = arc.new_collection(NodeType::Vertex, memory_size);
//             let memory_candidate_gate = arc.new_collection(NodeType::Vertex, memory_size);
//             let output_tahn_mul_gate = arc.new_collection(NodeType::Vertex, memory_size);
//             let tanh_gate = arc.new_collection(NodeType::Vertex, memory_size);

//             builder
//                 .one_to_many(&input, &input_to_forget_weights)
//                 .one_to_many(&input, &input_to_input_weights)
//                 .one_to_many(&input, &input_to_candidate_weights)
//                 .one_to_many(&input, &input_to_output_weights)
//                 .one_to_many(&output_tahn_mul_gate, &hidden_to_forget_weights)
//                 .one_to_many(&output_tahn_mul_gate, &hidden_to_input_weights)
//                 .one_to_many(&output_tahn_mul_gate, &hidden_to_candidate_weights)
//                 .one_to_many(&output_tahn_mul_gate, &hidden_to_output_weights)
//                 .many_to_one(&input_to_forget_weights, &forget_gate)
//                 .many_to_one(&hidden_to_forget_weights, &forget_gate)
//                 .many_to_one(&input_to_input_weights, &input_gate)
//                 .many_to_one(&hidden_to_input_weights, &input_gate)
//                 .many_to_one(&input_to_candidate_weights, &candidate_gate)
//                 .many_to_one(&hidden_to_candidate_weights, &candidate_gate)
//                 .many_to_one(&input_to_output_weights, &output_gate)
//                 .many_to_one(&hidden_to_output_weights, &output_gate)
//                 .one_to_one(&forget_gate, &forget_memory_mul_gate)
//                 .one_to_one(&memory_candidate_gate, &forget_memory_mul_gate)
//                 .one_to_one(&input_gate, &input_candidate_mul_gate)
//                 .one_to_one(&candidate_gate, &input_candidate_mul_gate)
//                 .one_to_one(&forget_memory_mul_gate, &memory_candidate_gate)
//                 .one_to_one(&input_candidate_mul_gate, &memory_candidate_gate)
//                 .one_to_one(&memory_candidate_gate, &tanh_gate)
//                 .one_to_one(&tanh_gate, &output_tahn_mul_gate)
//                 .one_to_one(&output_gate, &output_tahn_mul_gate)
//                 .one_to_many(&output_tahn_mul_gate, &output_weights)
//                 .many_to_one(&output_weights, &output)
//                 .build()
//         })
//     }

//     pub fn gru(&self, input_size: usize, output_size: usize, memory_size: usize) -> Graph<T> {
//         GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
//             let input = arc.input(input_size);
//             let output = arc.output(output_size);

//             let output_weights = arc.edge(memory_size * output_size);

//             let reset_gate = arc.vertex(memory_size);
//             let update_gate = arc.vertex(memory_size);
//             let candidate_gate = arc.vertex(memory_size);

//             let input_to_reset_weights = arc.edge(input_size * memory_size);
//             let input_to_update_weights = arc.edge(input_size * memory_size);
//             let input_to_candidate_weights = arc.edge(input_size * memory_size);

//             let hidden_to_reset_weights = arc.edge(memory_size * memory_size);
//             let hidden_to_update_weights = arc.edge(memory_size * memory_size);
//             let hidden_to_candidate_weights = arc.edge(memory_size * memory_size);

//             let hidden_reset_gate = arc.new_collection(NodeType::Vertex, memory_size);
//             let update_candidate_mul_gate = arc.new_collection(NodeType::Vertex, memory_size);
//             let invert_update_gate = arc.new_collection(NodeType::Vertex, memory_size);
//             let hidden_invert_mul_gate = arc.new_collection(NodeType::Vertex, memory_size);
//             let candidate_hidden_add_gate = arc.new_collection(NodeType::Vertex, memory_size);

//             builder
//                 .one_to_many(&input, &input_to_reset_weights)
//                 .one_to_many(&input, &input_to_update_weights)
//                 .one_to_many(&input, &input_to_candidate_weights)
//                 .one_to_many(&candidate_hidden_add_gate, &hidden_to_reset_weights)
//                 .one_to_many(&candidate_hidden_add_gate, &hidden_to_update_weights)
//                 .many_to_one(&input_to_reset_weights, &reset_gate)
//                 .many_to_one(&hidden_to_reset_weights, &reset_gate)
//                 .many_to_one(&input_to_update_weights, &update_gate)
//                 .many_to_one(&hidden_to_update_weights, &update_gate)
//                 .one_to_one(&reset_gate, &hidden_reset_gate)
//                 .one_to_one(&candidate_hidden_add_gate, &hidden_reset_gate)
//                 .one_to_many(&hidden_reset_gate, &hidden_to_candidate_weights)
//                 .many_to_one(&input_to_candidate_weights, &candidate_gate)
//                 .many_to_one(&hidden_to_candidate_weights, &candidate_gate)
//                 .one_to_one(&update_gate, &update_candidate_mul_gate)
//                 .one_to_one(&candidate_gate, &update_candidate_mul_gate)
//                 .one_to_one(&update_gate, &invert_update_gate)
//                 .one_to_one(&candidate_hidden_add_gate, &hidden_invert_mul_gate)
//                 .one_to_one(&invert_update_gate, &hidden_invert_mul_gate)
//                 .one_to_one(&hidden_invert_mul_gate, &candidate_hidden_add_gate)
//                 .one_to_one(&update_candidate_mul_gate, &candidate_hidden_add_gate)
//                 .one_to_many(&candidate_hidden_add_gate, &output_weights)
//                 .many_to_one(&output_weights, &output)
//                 .build()
//         })
//     }
// }

// pub enum GraphArchitecture {
//     Acyclic,
//     Cyclic,
//     WeightedAcyclic,
//     WeightedCyclic,
//     AttentionUnit,
//     Hopfield,
//     LSTM,
//     GRU,
// }

// pub struct GraphSchema<T> {
//     input_size: usize,
//     output_size: usize,
//     memory_size: usize,
//     vertecies: Vec<Operation<T>>,
//     edges: Vec<Operation<T>>,
//     outputs: Vec<Operation<T>>,
//     graph_type: GraphArchitecture,
// }

// impl GraphSchema<f32> {
//     pub fn dense(input_size: usize, output_size: usize) -> Self {
//         GraphSchema {
//             input_size,
//             output_size,
//             memory_size: 0,
//             vertecies: vec![operation::linear()],
//             edges: vec![operation::weight(), operation::identity()],
//             outputs: vec![operation::linear()],
//             graph_type: GraphArchitecture::Acyclic,
//         }
//     }

//     pub fn with_memory(mut self, memory_size: usize) -> Self {
//         self.memory_size = memory_size;
//         self
//     }

//     pub fn with_vertices(mut self, vertecies: Vec<Operation<f32>>) -> Self {
//         self.vertecies = vertecies;
//         self
//     }

//     pub fn with_edges(mut self, edges: Vec<Operation<f32>>) -> Self {
//         self.edges = edges;
//         self
//     }

//     pub fn with_outputs(mut self, outputs: Vec<Operation<f32>>) -> Self {
//         self.outputs = outputs;
//         self
//     }

//     pub fn input(&self, size: usize) -> Graph<f32> {
//         let nodes = (0..size)
//             .map(|i| GraphNode::new(i, NodeType::Input, operation::var(i)))
//             .collect::<Vec<GraphNode<f32>>>();

//         Graph::new(nodes)
//     }

//     pub fn output(&self, size: usize) -> Graph<f32> {
//         if self.outputs.is_empty() {
//             let nodes = (0..size)
//                 .map(|i| GraphNode::new(i, NodeType::Output, operation::linear()))
//                 .collect::<Vec<GraphNode<f32>>>();

//             return Graph::new(nodes);
//         } else {
//             let nodes = (0..size)
//                 .map(|i| {
//                     GraphNode::new(
//                         i,
//                         NodeType::Output,
//                         random_provider::choose(&self.outputs).new_instance(),
//                     )
//                 })
//                 .collect::<Vec<GraphNode<f32>>>();

//             return Graph::new(nodes);
//         }
//     }

//     pub fn vertex(&self, size: usize) -> Graph<f32> {
//         if self.vertecies.is_empty() {
//             let nodes = (0..size)
//                 .map(|i| GraphNode::new(i, NodeType::Vertex, operation::identity()))
//                 .collect::<Vec<GraphNode<f32>>>();

//             return Graph::new(nodes);
//         } else {
//             let nodes = (0..size)
//                 .map(|i| {
//                     GraphNode::new(
//                         i,
//                         NodeType::Vertex,
//                         random_provider::choose(&self.vertecies).new_instance(),
//                     )
//                 })
//                 .collect::<Vec<GraphNode<f32>>>();

//             return Graph::new(nodes);
//         }
//     }

//     pub fn edge(&self, size: usize) -> Graph<f32> {
//         if self.edges.is_empty() {
//             let nodes = (0..size)
//                 .map(|i| GraphNode::new(i, NodeType::Edge, operation::identity()))
//                 .collect::<Vec<GraphNode<f32>>>();

//             return Graph::new(nodes);
//         } else {
//             let nodes = (0..size)
//                 .map(|i| {
//                     GraphNode::new(
//                         i,
//                         NodeType::Edge,
//                         random_provider::choose(&self.edges).new_instance(),
//                     )
//                 })
//                 .collect::<Vec<GraphNode<f32>>>();

//             return Graph::new(nodes);
//         }
//     }
// }

// impl Builder for GraphSchema<f32> {
//     type Output = Graph<f32>;

//     fn build(&self) -> Self::Output {
//         match self.graph_type {
//             GraphArchitecture::Acyclic => AsyclicBuilder(self).build(),
//             _ => unimplemented!(),
//         }
//     }
// }

// struct AsyclicBuilder<'a, T>(&'a GraphSchema<T>);

// impl<'a> Builder for AsyclicBuilder<'a, f32> {
//     type Output = Graph<f32>;

//     fn build(&self) -> Self::Output {
//         let schema = &self.0;

//         let input = schema.input(schema.input_size);
//         let output = schema.output(schema.output_size);

//         let architect = GraphArchitect::default();

//         architect.all_to_all(&input, &output).build()
//     }
// }

// #[cfg(test)]
// mod test {
//     use radiate::Valid;

//     use crate::ops::operation;

//     use super::*;

//     #[test]
//     fn graph_builder_simple_acyclic_f32() {
//         let factory = NodeFactory::new()
//             .inputs((0..2).map(operation::var).collect())
//             .vertices(vec![operation::linear()])
//             .outputs(vec![operation::linear()]);

//         // let node = factory.generate((0, NodeType::Vertex));
//         let builder = GraphBuilder::<f32>::default()
//             .with_inputs(vec![operation::var(0), operation::var(1)])
//             .with_outputs(vec![operation::linear()]);

//         let graph = builder.acyclic(2, 1);

//         assert_eq!(graph.len(), 3);
//     }

//     #[test]
//     fn graph_builder_simple_cyclic_f32() {
//         let builder = GraphBuilder::<f32>::default()
//             .with_inputs(vec![operation::var(0), operation::var(1)])
//             .with_outputs(vec![operation::linear()]);

//         let graph = builder.cyclic(2, 1);

//         let other = GraphSchema::dense(2, 1).build();

//         println!("{:?}", graph);
//     }
// }
