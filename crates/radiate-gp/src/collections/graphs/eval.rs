use super::{Graph, GraphNode, iter::GraphIterator};
use crate::{Eval, EvalMut, NodeType, node::Node};
use std::ops::Range;

/// [GraphEvaluator] is a struct that is used to evaluate a [Graph] of [GraphNode]'s. It uses the [GraphIterator]
/// to traverse the [Graph] in a sudo-topological order and evaluate the nodes in the correct order.
///
/// On the first iteration it caches the order of nodes in the [Graph] and then uses that order to
/// evaluate the nodes in the correct order. This is a massive performance improvement.
pub struct GraphEvaluator<'a, T, V> {
    nodes: &'a [GraphNode<T>],
    eval_order: Vec<usize>,
    outputs: Vec<V>,
    inputs: Vec<V>,
    output_outs: Vec<V>,
    input_ranges: Vec<Range<usize>>,
}

impl<'a, T, V> GraphEvaluator<'a, T, V>
where
    T: Eval<[V], V>,
    V: Default + Clone,
{
    /// Creates a new [GraphEvaluator] with the given [Graph]. We pre-allocate a
    /// `Vec<Vec<V>>` to hold the inputs for each node and a `Vec<V>` to hold the outputs
    /// of nodes which serve as the inputs for their descendants. Then, iterating over
    /// the nodes in topological order, we evaluate the nodes in one pass. Because of this,
    /// the 'heavy lifting' is done upfront which makes subsequent evaluations much faster.
    ///
    /// # Arguments
    /// * graph - The [Graph] to reduce.
    pub fn new<N>(graph: &'a N) -> GraphEvaluator<'a, T, V>
    where
        N: AsRef<[GraphNode<T>]>,
    {
        let nodes = graph.as_ref();

        let mut input_ranges = Vec::with_capacity(nodes.len());
        let mut total_inputs = 0;

        for node in nodes {
            let input_len = node.incoming().len();
            input_ranges.push(total_inputs..total_inputs + input_len);
            total_inputs += input_len;
        }

        let output_size = nodes
            .iter()
            .filter(|node| node.node_type() == NodeType::Output)
            .count();

        let inputs = vec![V::default(); total_inputs];
        let output_outs = vec![V::default(); output_size];

        GraphEvaluator {
            nodes,
            inputs,
            output_outs,
            eval_order: nodes.iter_topological().map(|node| node.index()).collect(),
            outputs: vec![V::default(); nodes.len()],
            input_ranges,
        }
    }
}

/// Implements the `EvalMut` trait for [GraphEvaluator].
impl<T, V> EvalMut<[V], Vec<V>> for GraphEvaluator<'_, T, V>
where
    T: Eval<[V], V>,
    V: Clone + Default,
{
    /// Evaluates the [Graph] with the given input. Returns the output of the [Graph].
    /// The `eval` method will cache the order of nodes in the [Graph] on the first iteration.
    /// On subsequent iterations it will use the cached order to evaluate the [Graph] in the correct order.
    ///
    /// # Arguments
    /// * `input` - A `Vec` of `T` to evaluate the [Graph] with.
    ///
    ///  # Returns
    /// * A `Vec` of `T` which is the output of the [Graph].
    #[inline]
    fn eval_mut(&mut self, input: &[V]) -> Vec<V> {
        self.output_outs.truncate(0);

        for index in self.eval_order.iter() {
            let node = &self.nodes[*index];
            let incoming = node.incoming();
            if incoming.is_empty() {
                self.outputs[node.index()] = node.eval(input);
            } else {
                let input_range = &self.input_ranges[node.index()];
                let input_slice = &mut self.inputs[input_range.clone()];

                for (dst, incoming) in input_slice.iter_mut().zip(incoming) {
                    *dst = self.outputs[*incoming].clone();
                }

                self.outputs[node.index()] = node.eval(input_slice);
            }

            if node.node_type() == NodeType::Output {
                self.output_outs.push(self.outputs[node.index()].clone());
            }
        }

        self.output_outs.clone()
    }
}

impl<T, V> Eval<Vec<Vec<V>>, Vec<Vec<V>>> for Graph<T>
where
    T: Eval<[V], V>,
    V: Clone + Default,
{
    /// Evaluates the [Graph] with the given input 'Vec<Vec<T>>'. Returns the output of the [Graph] as 'Vec<Vec<T>>'.
    /// This is intended to be used when evaluating a batch of inputs.
    ///
    /// # Arguments
    /// * `input` - A `Vec<Vec<T>>` to evaluate the [Graph] with.
    ///
    /// # Returns
    /// * A `Vec<Vec<T>>` which is the output of the [Graph].
    #[inline]
    fn eval(&self, input: &Vec<Vec<V>>) -> Vec<Vec<V>> {
        let mut evaluator = GraphEvaluator::new(self);
        input
            .iter()
            .map(|input| evaluator.eval_mut(input))
            .collect()
    }
}

impl<T, V> Eval<[V], V> for GraphNode<T>
where
    T: Eval<[V], V>,
    V: Clone,
{
    /// Evaluates the [GraphNode]' with the given input. Returns the output of the [GraphNode].
    /// # Arguments
    /// * `inputs` - A `Vec` of `V` to evaluate the [GraphNode]' with.
    ///
    /// # Returns
    /// * A `V` which is the output of the [GraphNode]'.
    #[inline]
    fn eval(&self, inputs: &[V]) -> V {
        self.value().eval(inputs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Graph, Op};

    #[test]
    fn test_graph_eval_simple() {
        let mut graph = Graph::<Op<f32>>::default();

        let idx_one = graph.insert(NodeType::Input, Op::var(0));
        let idx_two = graph.insert(NodeType::Input, Op::constant(5_f32));
        let idx_three = graph.insert(NodeType::Vertex, Op::add());
        let idx_four = graph.insert(NodeType::Output, Op::linear());

        graph
            .attach(idx_one, idx_three)
            .attach(idx_two, idx_three)
            .attach(idx_three, idx_four);

        let six = graph.eval(&vec![vec![1_f32]]);
        let seven = graph.eval(&vec![vec![2_f32]]);
        let eight = graph.eval(&vec![vec![3_f32]]);

        assert_eq!(six, &[&[6_f32]]);
        assert_eq!(seven, &[&[7_f32]]);
        assert_eq!(eight, &[&[8_f32]]);
        assert_eq!(graph.len(), 4);
    }
}

// use super::{Graph, GraphNode, iter::GraphIterator};
// use crate::{Eval, EvalMut, NodeType, node::Node};

// /// [GraphEvaluator] is a struct that is used to evaluate a [Graph] of [GraphNode]'s. It uses the [GraphIterator]
// /// to traverse the [Graph] in a sudo-topological order and evaluate the nodes in the correct order.
// ///
// /// On the first iteration it caches the order of nodes in the [Graph] and then uses that order to
// /// evaluate the nodes in the correct order. This is a massive performance improvement.
// pub struct GraphEvaluator<'a, T, V> {
//     nodes: &'a [GraphNode<T>],
//     output_size: usize,
//     eval_order: Vec<usize>,
//     outputs: Vec<V>,
//     inputs: Vec<V>,
//     output_outs: Vec<V>,
//     // inputs: Vec<Vec<V>>,
//     input_ranges: Vec<std::ops::Range<usize>>,
// }

// impl<'a, T, V> GraphEvaluator<'a, T, V>
// where
//     T: Eval<[V], V>,
//     V: Default + Clone,
// {
//     /// Creates a new [GraphEvaluator] with the given [Graph]. We pre-allocate a
//     /// `Vec<Vec<V>>` to hold the inputs for each node and a `Vec<V>` to hold the outputs
//     /// of nodes which serve as the inputs for their descendants. Then, iterating over
//     /// the nodes in topological order, we evaluate the nodes in one pass. Because of this,
//     /// the 'heavy lifting' is done upfront which makes subsequent evaluations much faster.
//     ///
//     /// # Arguments
//     /// * graph - The [Graph] to reduce.
//     pub fn new<N>(graph: &'a N) -> GraphEvaluator<'a, T, V>
//     where
//         N: AsRef<[GraphNode<T>]>,
//     {
//         let nodes = graph.as_ref();

//         let mut input_ranges = Vec::with_capacity(nodes.len());
//         let mut total_inputs = 0;

//         for node in nodes {
//             let input_len = node.incoming().len();
//             input_ranges.push(total_inputs..total_inputs + input_len);
//             total_inputs += input_len;
//         }

//         let output_size = nodes
//             .iter()
//             .filter(|node| node.node_type() == NodeType::Output)
//             .count();

//         let inputs = vec![V::default(); total_inputs];
//         let output_outs = vec![V::default(); output_size];

//         GraphEvaluator {
//             nodes,
//             output_size: output_size,
//             inputs,
//             output_outs,
//             // inputs: nodes
//             //     .iter()
//             //     .map(|node| vec![V::default(); node.incoming().len()])
//             //     .collect::<Vec<Vec<V>>>(),
//             eval_order: nodes.iter_topological().map(|node| node.index()).collect(),
//             outputs: vec![V::default(); nodes.len()],
//             input_ranges,
//         }
//     }
// }

// /// Implements the `EvalMut` trait for [GraphEvaluator].
// impl<T, V> EvalMut<[V], Vec<V>> for GraphEvaluator<'_, T, V>
// where
//     T: Eval<[V], V>,
//     V: Clone + Default,
// {
//     /// Evaluates the [Graph] with the given input. Returns the output of the [Graph].
//     /// The `eval` method will cache the order of nodes in the [Graph] on the first iteration.
//     /// On subsequent iterations it will use the cached order to evaluate the [Graph] in the correct order.
//     ///
//     /// # Arguments
//     /// * `input` - A `Vec` of `T` to evaluate the [Graph] with.
//     ///
//     ///  # Returns
//     /// * A `Vec` of `T` which is the output of the [Graph].
//     #[inline]
//     fn eval_mut(&mut self, input: &[V]) -> Vec<V> {
//         self.output_outs.clear();
//         self.output_outs.reserve(self.output_size);

//         for index in self.eval_order.iter() {
//             let node = &self.nodes[*index];
//             if node.incoming().is_empty() {
//                 self.outputs[node.index()] = node.eval(input);
//             } else {
//                 let input_range = &self.input_ranges[node.index()];
//                 let input_slice = &mut self.inputs[input_range.clone()];

//                 for (dst, incoming) in input_slice.iter_mut().zip(node.incoming()) {
//                     *dst = self.outputs[*incoming].clone();
//                 }

//                 self.outputs[node.index()] = node.eval(input_slice);
//                 // for (idx, incoming) in node.incoming().iter().enumerate() {
//                 //     self.inputs[node.index()][idx] = self.outputs[*incoming].clone();
//                 // }

//                 // self.outputs[node.index()] = node.eval(&self.inputs[node.index()]);
//             }

//             if node.node_type() == NodeType::Output {
//                 self.output_outs.push(self.outputs[node.index()].clone());
//             }
//         }

//         self.output_outs.clone()
//     }
// }

// impl<T, V> Eval<Vec<Vec<V>>, Vec<Vec<V>>> for Graph<T>
// where
//     T: Eval<[V], V>,
//     V: Clone + Default,
// {
//     /// Evaluates the [Graph] with the given input 'Vec<Vec<T>>'. Returns the output of the [Graph] as 'Vec<Vec<T>>'.
//     /// This is intended to be used when evaluating a batch of inputs.
//     ///
//     /// # Arguments
//     /// * `input` - A `Vec<Vec<T>>` to evaluate the [Graph] with.
//     ///
//     /// # Returns
//     /// * A `Vec<Vec<T>>` which is the output of the [Graph].
//     #[inline]
//     fn eval(&self, input: &Vec<Vec<V>>) -> Vec<Vec<V>> {
//         let mut evaluator = GraphEvaluator::new(self);
//         input
//             .iter()
//             .map(|input| evaluator.eval_mut(input))
//             .collect()
//     }
// }

// impl<T, V> Eval<[V], V> for GraphNode<T>
// where
//     T: Eval<[V], V>,
//     V: Clone,
// {
//     /// Evaluates the [GraphNode]' with the given input. Returns the output of the [GraphNode].
//     /// # Arguments
//     /// * `inputs` - A `Vec` of `V` to evaluate the [GraphNode]' with.
//     ///
//     /// # Returns
//     /// * A `V` which is the output of the [GraphNode]'.
//     #[inline]
//     fn eval(&self, inputs: &[V]) -> V {
//         self.value().eval(inputs)
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{Graph, Op};

//     #[test]
//     fn test_graph_eval_simple() {
//         let mut graph = Graph::<Op<f32>>::default();

//         let idx_one = graph.insert(NodeType::Input, Op::var(0));
//         let idx_two = graph.insert(NodeType::Input, Op::constant(5_f32));
//         let idx_three = graph.insert(NodeType::Vertex, Op::add());
//         let idx_four = graph.insert(NodeType::Output, Op::linear());

//         graph
//             .attach(idx_one, idx_three)
//             .attach(idx_two, idx_three)
//             .attach(idx_three, idx_four);

//         let six = graph.eval(&vec![vec![1_f32]]);
//         let seven = graph.eval(&vec![vec![2_f32]]);
//         let eight = graph.eval(&vec![vec![3_f32]]);

//         assert_eq!(six, &[&[6_f32]]);
//         assert_eq!(seven, &[&[7_f32]]);
//         assert_eq!(eight, &[&[8_f32]]);
//         assert_eq!(graph.len(), 4);
//     }
// }
