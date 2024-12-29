use crate::architect::GraphArchitect;
use crate::operation::Operation;
use crate::{operation, Graph, GraphNode, NodeType, OpStore, Tree, TreeNode};

use radiate::random_provider;

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

#[derive(Default)]
pub struct GraphBuilder<T: Clone + Default> {
    node_factory: OpStore<T>,
}

impl<T: Clone + Default> GraphBuilder<T> {
    pub fn new(node_factory: &OpStore<T>) -> Self {
        GraphBuilder {
            node_factory: node_factory.clone(),
        }
    }

    pub fn build<F>(&self, build_fn: F) -> Graph<T>
    where
        F: FnOnce(&GraphBuilder<T>, GraphArchitect<T>) -> Graph<T>,
    {
        build_fn(self, GraphArchitect::new(&self.node_factory))
    }

    pub fn input(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Input, size)
    }

    pub fn output(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Output, size)
    }

    pub fn vertex(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Vertex, size)
    }

    pub fn edge(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Edge, size)
    }

    pub fn new_collection(&self, node_type: NodeType, size: usize) -> Graph<T> {
        let nodes = self.new_nodes(node_type, size);
        Graph::new(nodes)
    }

    pub fn new_nodes(&self, node_type: NodeType, size: usize) -> Vec<GraphNode<T>> {
        (0..size)
            .map(|i| self.node_factory.new_node(i, node_type))
            .collect::<Vec<GraphNode<T>>>()
    }

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
        self.node_factory.add_node_values(node_type, values);
    }
}

impl<T> GraphBuilder<T>
where
    T: Clone + PartialEq + Default,
{
    pub fn acyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
        GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
            builder
                .all_to_all(&arc.input(input_size), &arc.output(output_size))
                .build()
        })
    }

    pub fn cyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
        GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let aggregate = arc.vertex(input_size);
            let link = arc.vertex(input_size);
            let output = arc.output(output_size);

            builder
                .one_to_one(&input, &aggregate)
                .one_to_one_self(&aggregate, &link)
                .all_to_all(&aggregate, &output)
                .build()
        })
    }

    pub fn weighted_acyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
        GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);
            let weights = arc.edge(input_size * output_size);

            builder
                .one_to_many(&input, &weights)
                .many_to_one(&weights, &output)
                .build()
        })
    }

    pub fn weighted_cyclic(
        &self,
        input_size: usize,
        output_size: usize,
        memory_size: usize,
    ) -> Graph<T> {
        GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);
            let weights = arc.edge(input_size * memory_size);
            let aggregate = arc.vertex(memory_size);
            let aggregate_weights = arc.edge(memory_size);

            builder
                .one_to_many(&input, &weights)
                .many_to_one(&weights, &aggregate)
                .one_to_one_self(&aggregate, &aggregate_weights)
                .all_to_all(&aggregate, &output)
                .build()
        })
    }

    pub fn attention_unit(
        &self,
        input_size: usize,
        output_size: usize,
        num_heads: usize,
    ) -> Graph<T> {
        GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);

            let query_weights = arc.edge(input_size * num_heads);
            let key_weights = arc.edge(input_size * num_heads);
            let value_weights = arc.edge(input_size * num_heads);

            let attention_scores = arc.new_collection(NodeType::Vertex, num_heads);
            let attention_aggreg = arc.new_collection(NodeType::Vertex, num_heads);

            builder
                .one_to_many(&input, &query_weights)
                .one_to_many(&input, &key_weights)
                .one_to_many(&input, &value_weights)
                .many_to_one(&query_weights, &attention_scores)
                .many_to_one(&key_weights, &attention_scores)
                .one_to_many(&attention_scores, &attention_aggreg)
                .many_to_one(&value_weights, &attention_aggreg)
                .many_to_one(&attention_aggreg, &output)
                .build()
        })
    }

    pub fn hopfield(&self, input_size: usize, output_size: usize) -> Graph<T> {
        GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);
            let aggregates = arc.vertex(input_size);
            let weights = arc.edge(input_size * output_size);

            builder
                .one_to_many(&input, &aggregates)
                .one_to_many(&aggregates, &weights)
                .many_to_one(&weights, &aggregates)
                .many_to_one(&aggregates, &output)
                .build()
        })
    }

    pub fn lstm(&self, input_size: usize, output_size: usize, memory_size: usize) -> Graph<T> {
        GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);

            let input_to_forget_weights = arc.edge(input_size * memory_size);
            let hidden_to_forget_weights = arc.edge(memory_size * memory_size);

            let input_to_input_weights = arc.edge(input_size * memory_size);
            let hidden_to_input_weights = arc.edge(memory_size * memory_size);

            let input_to_candidate_weights = arc.edge(input_size * memory_size);
            let hidden_to_candidate_weights = arc.edge(memory_size * memory_size);

            let input_to_output_weights = arc.edge(input_size * memory_size);
            let hidden_to_output_weights = arc.edge(memory_size * memory_size);

            let output_weights = arc.edge(memory_size * output_size);

            let forget_gate = arc.vertex(memory_size);
            let input_gate = arc.vertex(memory_size);
            let candidate_gate = arc.vertex(memory_size);
            let output_gate = arc.vertex(memory_size);

            let input_candidate_mul_gate = arc.new_collection(NodeType::Vertex, memory_size);
            let forget_memory_mul_gate = arc.new_collection(NodeType::Vertex, memory_size);
            let memory_candidate_gate = arc.new_collection(NodeType::Vertex, memory_size);
            let output_tahn_mul_gate = arc.new_collection(NodeType::Vertex, memory_size);
            let tanh_gate = arc.new_collection(NodeType::Vertex, memory_size);

            builder
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
        })
    }

    pub fn gru(&self, input_size: usize, output_size: usize, memory_size: usize) -> Graph<T> {
        GraphBuilder::<T>::new(&self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);

            let output_weights = arc.edge(memory_size * output_size);

            let reset_gate = arc.vertex(memory_size);
            let update_gate = arc.vertex(memory_size);
            let candidate_gate = arc.vertex(memory_size);

            let input_to_reset_weights = arc.edge(input_size * memory_size);
            let input_to_update_weights = arc.edge(input_size * memory_size);
            let input_to_candidate_weights = arc.edge(input_size * memory_size);

            let hidden_to_reset_weights = arc.edge(memory_size * memory_size);
            let hidden_to_update_weights = arc.edge(memory_size * memory_size);
            let hidden_to_candidate_weights = arc.edge(memory_size * memory_size);

            let hidden_reset_gate = arc.new_collection(NodeType::Vertex, memory_size);
            let update_candidate_mul_gate = arc.new_collection(NodeType::Vertex, memory_size);
            let invert_update_gate = arc.new_collection(NodeType::Vertex, memory_size);
            let hidden_invert_mul_gate = arc.new_collection(NodeType::Vertex, memory_size);
            let candidate_hidden_add_gate = arc.new_collection(NodeType::Vertex, memory_size);

            builder
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
        })
    }
}

impl GraphBuilder<f32> {
    pub fn dense(input_size: usize, output_size: usize, activation: Operation<f32>) -> Graph<f32> {
        let factory = OpStore::new()
            .inputs((0..input_size).map(operation::var).collect())
            .edges(vec![operation::weight()])
            .outputs(vec![activation]);

        GraphBuilder::new(&factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);
            let weights = arc.edge(input_size * output_size);

            builder
                .one_to_many(&input, &weights)
                .many_to_one(&weights, &output)
                .build()
        })
    }

    pub fn recurrent(
        input_size: usize,
        output_size: usize,
        memory_size: usize,
        activation: Operation<f32>,
    ) -> Graph<f32> {
        let factory = OpStore::new()
            .inputs((0..input_size).map(operation::var).collect())
            .edges(vec![operation::weight()])
            .vertices(vec![activation.clone()])
            .outputs(vec![activation]);

        GraphBuilder::new(&factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);
            let weights = arc.edge(input_size * memory_size);
            let aggregate = arc.vertex(memory_size);
            let aggregate_weights = arc.edge(memory_size);

            builder
                .one_to_many(&input, &weights)
                .many_to_one(&weights, &aggregate)
                .one_to_one_self(&aggregate, &aggregate_weights)
                .all_to_all(&aggregate, &output)
                .build()
        })
    }
}

#[cfg(test)]
mod test {
    use radiate::Valid;

    use crate::operation;

    use super::*;

    #[test]
    fn graph_builder_simple_acyclic_f32() {
        let builder = GraphBuilder::<f32>::default()
            .with_inputs(vec![operation::var(0), operation::var(1)])
            .with_outputs(vec![operation::linear()]);

        let graph = builder.acyclic(2, 1);

        assert_eq!(graph.get_nodes().len(), 3);
    }

    #[test]
    fn graph_builder_simple_cyclic_f32() {
        let builder = GraphBuilder::<f32>::default()
            .with_inputs(vec![operation::var(0), operation::var(1)])
            .with_outputs(vec![operation::linear()]);

        let graph = builder.cyclic(2, 1);

        println!("{:?}", graph);
    }

    #[test]
    fn graph_builder_dense_produces_valid_graph() {
        let graph = GraphBuilder::<f32>::dense(2, 1, operation::linear());

        assert!(graph.len() == 5);
        assert!(graph.is_valid());
    }

    #[test]
    fn graph_builder_recurrent_produces_valid_graph() {
        let graph = GraphBuilder::<f32>::recurrent(2, 1, 3, operation::linear());

        println!("{:?}", graph);

        // assert!(graph.len() == 7);
        // assert!(graph.is_valid());
    }
}
