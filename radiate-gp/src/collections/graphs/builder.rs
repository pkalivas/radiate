use super::aggregate::GraphAggregate;
use super::store::NodeStore;
use crate::collections::{Builder, Graph, GraphNode, NodeType};
use crate::Factory;
use std::sync::Arc;

pub struct AsyclicGraphBuilder<T> {
    input_size: usize,
    output_size: usize,
    inner: NodeBuilder<T>,
}

impl<T: Clone + Default> AsyclicGraphBuilder<T> {
    pub fn new(input_size: usize, output_size: usize, values: impl Into<NodeBuilder<T>>) -> Self {
        AsyclicGraphBuilder {
            input_size,
            output_size,
            inner: values.into(),
        }
    }
}

impl<T: Clone + Default> Builder for AsyclicGraphBuilder<T> {
    type Output = Graph<T>;

    fn build(&self) -> Self::Output {
        let input_nodes = self.inner.input(self.input_size);
        let output_nodes = self.inner.output(self.output_size);

        GraphAggregate::new()
            .all_to_all(&input_nodes, &output_nodes)
            .build()
    }
}

pub struct NodeBuilder<T> {
    factory: Arc<dyn NodeStore<T>>,
}

impl<T: Clone + Default> NodeBuilder<T> {
    pub fn new(factory: Arc<dyn NodeStore<T>>) -> Self {
        NodeBuilder { factory }
    }

    pub fn input(&self, size: usize) -> Vec<GraphNode<T>> {
        self.new_nodes(NodeType::Input, size)
    }

    pub fn output(&self, size: usize) -> Vec<GraphNode<T>> {
        self.new_nodes(NodeType::Output, size)
    }

    pub fn edge(&self, size: usize) -> Vec<GraphNode<T>> {
        self.new_nodes(NodeType::Edge, size)
    }

    pub fn vertex(&self, size: usize) -> Vec<GraphNode<T>> {
        self.new_nodes(NodeType::Vertex, size)
    }

    pub fn new_nodes(&self, node_type: NodeType, size: usize) -> Vec<GraphNode<T>> {
        (0..size)
            .map(|i| self.factory.new_instance((i, node_type)))
            .collect::<Vec<GraphNode<T>>>()
    }
}

impl<T: Clone + Default> Factory<GraphNode<T>> for NodeBuilder<T> {
    type Input = (usize, NodeType);

    fn new_instance(&self, input: Self::Input) -> GraphNode<T> {
        self.factory.new_instance(input)
    }
}

impl<T> Clone for NodeBuilder<T> {
    fn clone(&self) -> Self {
        NodeBuilder {
            factory: Arc::clone(&self.factory),
        }
    }
}

// pub struct CyclicGraphBuilder<T> {
//     input_size: usize,
//     output_size: usize,
//     inner: NodeBuilder<T>,
// }

// impl<T: Clone + Default> CyclicGraphBuilder<T> {
//     pub fn new(input_size: usize, output_size: usize, values: impl Into<NodeStore<T>>) -> Self {
//         CyclicGraphBuilder {
//             input_size,
//             output_size,
//             inner: NodeBuilder::new(values),
//         }
//     }
// }

// impl<T: Clone + Default> Builder for CyclicGraphBuilder<T> {
//     type Output = Graph<T>;

//     fn build(&self) -> Self::Output {
//         let input = self.inner.input(self.input_size);
//         let aggregate = self.inner.vertex(self.input_size);
//         let link = self.inner.vertex(self.input_size);
//         let output = self.inner.output(self.output_size);

//         GraphAggregate::new()
//             .one_to_one(&input, &aggregate)
//             .one_to_one_self(&aggregate, &link)
//             .all_to_all(&aggregate, &output)
//             .build()
//     }
// }

#[cfg(test)]
mod tests {
    use crate::{
        ops::{self},
        Op,
    };

    use super::*;

    #[test]
    fn test_graph_builder() {
        let ops = ops::get_all_operations();
        let edges = vec![Op::weight(), Op::identity()];
        let outputs = vec![Op::sigmoid()];

        let store = ops
            .iter()
            .chain((0..4).map(Op::var).collect::<Vec<_>>().iter())
            .chain(edges.iter())
            .chain(outputs.iter())
            .cloned()
            .collect::<Vec<Op<f32>>>();

        let builder = AsyclicGraphBuilder::new(3, 3, store);
        let graph = builder.build();

        for node in graph.iter() {
            println!("{:?}", node);
        }

        let others = vec!["".to_string(), "".to_string(), "".to_string()];
        let vert = vec!["v1".to_string(), "v2".to_string(), "v3".to_string()];
        let edges = vec!["e1".to_string(), "e2".to_string(), "e3".to_string()];
        let outputs = vec!["o1".to_string(), "o2".to_string(), "o3".to_string()];

        let store2 = vec![
            (NodeType::Input, others.clone()),
            (NodeType::Output, outputs.clone()),
            (NodeType::Edge, edges.clone()),
            (NodeType::Vertex, vert.clone()),
        ];

        let builder =
            AsyclicGraphBuilder::<String>::new(3, 3, NodeBuilder::new(Arc::new(store2.into())));
        let graph = builder.build();

        for node in graph.iter() {
            println!("{:?}", node);
        }
    }

    // pub fn aggregates(&self, size: usize) -> Vec<GraphNode<T>> {
    //     let vertecies_with_any = self.store.values_with_arities(NodeType::Vertex, Arity::Any);
    //     let outputs_with_any = self.store.values_with_arities(NodeType::Output, Arity::Any);

    //     if !vertecies_with_any.is_empty() {
    //         return (0..size)
    //             .map(|i| {
    //                 let op = random_provider::choose(&vertecies_with_any).new_instance(());
    //                 GraphNode::new(i, NodeType::Vertex, op)
    //             })
    //             .collect::<Vec<GraphNode<T>>>();
    //     } else if !outputs_with_any.is_empty() {
    //         return (0..size)
    //             .map(|i| {
    //                 let op = random_provider::choose(&outputs_with_any).new_instance(());
    //                 GraphNode::new(i, NodeType::Vertex, op)
    //             })
    //             .collect::<Vec<GraphNode<T>>>();
    //     }

    //     self.new_nodes(NodeType::Vertex, size)
    // }

    // #[test]
    // fn test_acyclic_graph_builder() {
    //     let builder = AsyclicGraphBuilder::<f32>::new(3, 3, Op::sigmoid());
    //     let graph = builder.build();
    // }
}

// pub fn weighted_acyclic(
//     mut self,
//     input_size: usize,
//     output_size: usize,
//     output: Op<f32>,
// ) -> GraphBuilder<f32> {
//     self.with_values(NodeType::Input, (0..input_size).map(Op::var).collect());
//     self.with_values(NodeType::Output, vec![output]);

//     let input = self.input(input_size);
//     let output = self.output(output_size);
//     let weights = self.edge(input_size * output_size);

//     let graph = GraphAggregate::new()
//         .one_to_many(&input, &weights)
//         .many_to_one(&weights, &output)
//         .build();

//     self.node_cache = Some(graph.into_iter().collect());
//     self
// }

// pub fn weighted_cyclic(
//     mut self,
//     input_size: usize,
//     output_size: usize,
//     memory_size: usize,
//     output: Op<f32>,
// ) -> GraphBuilder<f32> {
//     self.with_values(NodeType::Input, (0..input_size).map(Op::var).collect());
//     self.with_values(NodeType::Output, vec![output]);

//     let input = self.input(input_size);
//     let output = self.output(output_size);
//     let weights = self.edge(input_size * memory_size);
//     let aggregate = self.aggregates(memory_size);
//     let aggregate_weights = self.edge(memory_size);

//     let graph = GraphAggregate::new()
//         .one_to_many(&input, &weights)
//         .many_to_one(&weights, &aggregate)
//         .one_to_one_self(&aggregate, &aggregate_weights)
//         .all_to_all(&aggregate, &output)
//         .build();

//     self.node_cache = Some(graph.into_iter().collect());
//     self
// }

// pub fn gru(
//     mut self,
//     input_size: usize,
//     output_size: usize,
//     memory_size: usize,
//     output: Op<f32>,
// ) -> GraphBuilder<f32> {
//     self.with_values(NodeType::Input, (0..input_size).map(Op::var).collect());
//     self.with_values(NodeType::Output, vec![output]);

//     let input = self.input(input_size);
//     let output = self.output(output_size);

//     let output_weights = self.edge(memory_size * output_size);

//     let reset_gate = self.aggregates(memory_size);
//     let update_gate = self.aggregates(memory_size);
//     let candidate_gate = self.aggregates(memory_size);

//     let input_to_reset_weights = self.edge(input_size * memory_size);
//     let input_to_update_weights = self.edge(input_size * memory_size);
//     let input_to_candidate_weights = self.edge(input_size * memory_size);

//     let hidden_to_reset_weights = self.edge(memory_size * memory_size);
//     let hidden_to_update_weights = self.edge(memory_size * memory_size);
//     let hidden_to_candidate_weights = self.edge(memory_size * memory_size);

//     let hidden_reset_gate = self.aggregates(memory_size);
//     let update_candidate_mul_gate = self.aggregates(memory_size);
//     let invert_update_gate = self.aggregates(memory_size);
//     let hidden_invert_mul_gate = self.aggregates(memory_size);
//     let candidate_hidden_add_gate = self.aggregates(memory_size);

//     let graph = GraphAggregate::new()
//         .one_to_many(&input, &input_to_reset_weights)
//         .one_to_many(&input, &input_to_update_weights)
//         .one_to_many(&input, &input_to_candidate_weights)
//         .one_to_many(&candidate_hidden_add_gate, &hidden_to_reset_weights)
//         .one_to_many(&candidate_hidden_add_gate, &hidden_to_update_weights)
//         .many_to_one(&input_to_reset_weights, &reset_gate)
//         .many_to_one(&hidden_to_reset_weights, &reset_gate)
//         .many_to_one(&input_to_update_weights, &update_gate)
//         .many_to_one(&hidden_to_update_weights, &update_gate)
//         .one_to_one(&reset_gate, &hidden_reset_gate)
//         .one_to_one(&candidate_hidden_add_gate, &hidden_reset_gate)
//         .one_to_many(&hidden_reset_gate, &hidden_to_candidate_weights)
//         .many_to_one(&input_to_candidate_weights, &candidate_gate)
//         .many_to_one(&hidden_to_candidate_weights, &candidate_gate)
//         .one_to_one(&update_gate, &update_candidate_mul_gate)
//         .one_to_one(&candidate_gate, &update_candidate_mul_gate)
//         .one_to_one(&update_gate, &invert_update_gate)
//         .one_to_one(&candidate_hidden_add_gate, &hidden_invert_mul_gate)
//         .one_to_one(&invert_update_gate, &hidden_invert_mul_gate)
//         .one_to_one(&hidden_invert_mul_gate, &candidate_hidden_add_gate)
//         .one_to_one(&update_candidate_mul_gate, &candidate_hidden_add_gate)
//         .one_to_many(&candidate_hidden_add_gate, &output_weights)
//         .many_to_one(&output_weights, &output)
//         .build();

//     self.node_cache = Some(graph.into_iter().collect());
//     self
// }

// pub fn lstm(mut self, input_size: usize, output_size: usize, output: Op<f32>) -> GraphBuilder<f32> {
//     self.with_values(NodeType::Input, (0..input_size).map(Op::var).collect());
//     self.with_values(NodeType::Output, vec![output]);

//     let input = self.input(input_size);
//     let output = self.output(output_size);

//     let cell_state = self.aggregates(1);
//     let hidden_state = self.aggregates(1);

//     let forget_gate = self.aggregates(1);
//     let input_gate = self.aggregates(1);
//     let output_gate = self.aggregates(1);
//     let candidate = self.aggregates(1);

//     let input_to_forget_weights = self.edge(input_size);
//     let input_to_input_weights = self.edge(input_size);
//     let input_to_output_weights = self.edge(input_size);
//     let input_to_candidate_weights = self.edge(input_size);

//     let hidden_to_forget_weights = self.edge(1);
//     let hidden_to_input_weights = self.edge(1);
//     let hidden_to_output_weights = self.edge(1);
//     let hidden_to_candidate_weights = self.edge(1);

//     let final_weights = self.edge(output_size);

//     let graph = GraphAggregate::new()
//         .one_to_one(&input, &input_to_forget_weights)
//         .one_to_one(&input, &input_to_input_weights)
//         .one_to_one(&input, &input_to_output_weights)
//         .one_to_one(&input, &input_to_candidate_weights)
//         .one_to_one(&hidden_state, &hidden_to_forget_weights)
//         .one_to_one(&hidden_state, &hidden_to_input_weights)
//         .one_to_one(&hidden_state, &hidden_to_output_weights)
//         .one_to_one(&hidden_state, &hidden_to_candidate_weights)
//         .many_to_one(&input_to_forget_weights, &forget_gate)
//         .many_to_one(&hidden_to_forget_weights, &forget_gate)
//         .many_to_one(&input_to_input_weights, &input_gate)
//         .many_to_one(&hidden_to_input_weights, &input_gate)
//         .many_to_one(&input_to_output_weights, &output_gate)
//         .many_to_one(&hidden_to_output_weights, &output_gate)
//         .many_to_one(&input_to_candidate_weights, &candidate)
//         .many_to_one(&hidden_to_candidate_weights, &candidate)
//         .one_to_one(&forget_gate, &cell_state)
//         .one_to_one(&input_gate, &candidate)
//         .one_to_one(&candidate, &cell_state)
//         .one_to_one(&cell_state, &hidden_state)
//         .one_to_one(&output_gate, &hidden_state)
//         .one_to_many(&hidden_state, &final_weights)
//         .one_to_one(&final_weights, &output)
//         .build();

//     self.node_cache = Some(graph.into_iter().collect());
//     self
// }
