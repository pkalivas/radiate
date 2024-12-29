use super::{Graph, GraphNode};
use crate::architects::builder::GraphBuilder;
use crate::architects::node_collections::node_factory::NodeFactory;
use crate::expr::Operation;
use crate::{NodeType, Tree, TreeNode};
use radiate::random_provider;

pub trait Archit {
    type Output;
    fn build(&self) -> Self::Output;
}

pub struct TreeArchit<T: Clone> {
    gates: Vec<Operation<T>>,
    leafs: Vec<Operation<T>>,
    depth: usize,
}

impl<T: Clone> TreeArchit<T> {
    pub fn new(depth: usize) -> Self {
        TreeArchit {
            gates: Vec::new(),
            leafs: Vec::new(),
            depth,
        }
    }

    pub fn gates(mut self, gates: Vec<Operation<T>>) -> Self {
        self.gates = gates;
        self
    }

    pub fn leafs(mut self, leafs: Vec<Operation<T>>) -> Self {
        self.leafs = leafs;
        self
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

impl<T: Clone + Default> Archit for TreeArchit<T> {
    type Output = Tree<T>;

    fn build(&self) -> Self::Output {
        let root = self.grow_tree(self.depth);
        Tree::new(root)
    }
}

pub struct Architect<'a, T>
where
    T: Clone + PartialEq + Default,
{
    node_factory: &'a NodeFactory<T>,
}

impl<'a, T> Architect<'a, T>
where
    T: Clone + PartialEq + Default,
{
    pub fn new(node_factory: &'a NodeFactory<T>) -> Self {
        Architect { node_factory }
    }

    pub fn build<F>(&self, build_fn: F) -> Graph<T>
    where
        F: FnOnce(&Architect<T>, GraphBuilder<T>) -> Graph<T>,
    {
        build_fn(self, GraphBuilder::new(self.node_factory))
    }

    pub fn input(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Input, size)
    }

    pub fn output(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Output, size)
    }

    pub fn gate(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Gate, size)
    }

    pub fn aggregate(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Aggregate, size)
    }

    pub fn weight(&self, size: usize) -> Graph<T> {
        self.new_collection(NodeType::Weight, size)
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
}

impl<T> Architect<'_, T>
where
    T: Clone + PartialEq + Default,
{
    pub fn acyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
        Architect::<T>::new(self.node_factory).build(|arc, builder| {
            builder
                .all_to_all(&arc.input(input_size), &arc.output(output_size))
                .build()
        })
    }

    pub fn cyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
        Architect::<T>::new(self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let aggregate = arc.aggregate(input_size);
            let link = arc.gate(input_size);
            let output = arc.output(output_size);

            builder
                .one_to_one(&input, &aggregate)
                .one_to_one_self(&aggregate, &link)
                .all_to_all(&aggregate, &output)
                .build()
        })
    }

    pub fn weighted_acyclic(&self, input_size: usize, output_size: usize) -> Graph<T> {
        Architect::<T>::new(self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);
            let weights = arc.weight(input_size * output_size);

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
        Architect::<T>::new(self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);
            let weights = arc.weight(input_size * memory_size);
            let aggregate = arc.aggregate(memory_size);
            let aggregate_weights = arc.weight(memory_size);

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
        Architect::<T>::new(self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);

            let query_weights = arc.weight(input_size * num_heads);
            let key_weights = arc.weight(input_size * num_heads);
            let value_weights = arc.weight(input_size * num_heads);

            let attention_scores = arc.new_collection(NodeType::Aggregate, num_heads);
            let attention_aggreg = arc.new_collection(NodeType::Aggregate, num_heads);

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
        Architect::<T>::new(self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);
            let aggregates = arc.aggregate(input_size);
            let weights = arc.weight(input_size * output_size);

            builder
                .one_to_many(&input, &aggregates)
                .one_to_many(&aggregates, &weights)
                .many_to_one(&weights, &aggregates)
                .many_to_one(&aggregates, &output)
                .build()
        })
    }

    pub fn lstm(&self, input_size: usize, output_size: usize, memory_size: usize) -> Graph<T> {
        Architect::<T>::new(self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);

            let input_to_forget_weights = arc.weight(input_size * memory_size);
            let hidden_to_forget_weights = arc.weight(memory_size * memory_size);

            let input_to_input_weights = arc.weight(input_size * memory_size);
            let hidden_to_input_weights = arc.weight(memory_size * memory_size);

            let input_to_candidate_weights = arc.weight(input_size * memory_size);
            let hidden_to_candidate_weights = arc.weight(memory_size * memory_size);

            let input_to_output_weights = arc.weight(input_size * memory_size);
            let hidden_to_output_weights = arc.weight(memory_size * memory_size);

            let output_weights = arc.weight(memory_size * output_size);

            let forget_gate = arc.aggregate(memory_size);
            let input_gate = arc.aggregate(memory_size);
            let candidate_gate = arc.aggregate(memory_size);
            let output_gate = arc.aggregate(memory_size);

            let input_candidate_mul_gate = arc.new_collection(NodeType::Aggregate, memory_size);
            let forget_memory_mul_gate = arc.new_collection(NodeType::Aggregate, memory_size);
            let memory_candidate_gate = arc.new_collection(NodeType::Aggregate, memory_size);
            let output_tahn_mul_gate = arc.new_collection(NodeType::Aggregate, memory_size);
            let tanh_gate = arc.new_collection(NodeType::Aggregate, memory_size);

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
        Architect::<T>::new(self.node_factory).build(|arc, builder| {
            let input = arc.input(input_size);
            let output = arc.output(output_size);

            let output_weights = arc.weight(memory_size * output_size);

            let reset_gate = arc.aggregate(memory_size);
            let update_gate = arc.aggregate(memory_size);
            let candidate_gate = arc.aggregate(memory_size);

            let input_to_reset_weights = arc.weight(input_size * memory_size);
            let input_to_update_weights = arc.weight(input_size * memory_size);
            let input_to_candidate_weights = arc.weight(input_size * memory_size);

            let hidden_to_reset_weights = arc.weight(memory_size * memory_size);
            let hidden_to_update_weights = arc.weight(memory_size * memory_size);
            let hidden_to_candidate_weights = arc.weight(memory_size * memory_size);

            let hidden_reset_gate = arc.new_collection(NodeType::Aggregate, memory_size);
            let update_candidate_mul_gate = arc.new_collection(NodeType::Aggregate, memory_size);
            let invert_update_gate = arc.new_collection(NodeType::Aggregate, memory_size);
            let hidden_invert_mul_gate = arc.new_collection(NodeType::Aggregate, memory_size);
            let candidate_hidden_add_gate = arc.new_collection(NodeType::Aggregate, memory_size);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr;

    #[test]
    fn test_tree_archit() {
        let tree_archit = TreeArchit::<f32>::new(3)
            .gates(vec![expr::add(), expr::sub()])
            .leafs(vec![expr::var(0), expr::var(1)]);
        let tree = tree_archit.build();
        let size = tree.root().map(|n| n.size()).unwrap_or(0);

        assert_eq!(size, 15);
    }
}
