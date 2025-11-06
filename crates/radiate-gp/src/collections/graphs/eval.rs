use super::{Graph, GraphNode, iter::GraphIterator};
use crate::{Eval, EvalMut, NodeType, eval::EvalIntoMut, node::Node};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::ops::Range;

/// A cache for storing intermediate results during graph evaluation.
///
/// This cache is used to store the inputs and outputs of each node in the graph
/// during evaluation, allowing for more efficient re-evaluation of nodes when
/// their inputs change. If we want to save a graph's evaluation between different evals,
/// we need to keep track of the inputs and outputs from previous runs incase of recurrent
/// structures. This cache is the answer to that.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GraphEvalCache<V> {
    eval_order: Vec<usize>,
    outputs: Vec<V>,
    inputs: Vec<V>,
    input_ranges: Vec<Range<usize>>,
    output_indices: SmallVec<[usize; 8]>,
}

/// [GraphEvaluator] is a struct that is used to evaluate a [Graph] of [GraphNode]'s. It uses the [GraphIterator]
/// to traverse the [Graph] in a pseudo-topological order and evaluate the nodes in the correct order.
pub struct GraphEvaluator<'a, T, V> {
    nodes: &'a [GraphNode<T>],
    inner: GraphEvalCache<V>,
}

impl<'a, T, V> GraphEvaluator<'a, T, V>
where
    T: Eval<[V], V>,
    V: Default + Clone,
{
    /// Creates a new [GraphEvaluator] with the given [Graph]. We pre-allocate the necessary
    /// storage for inputs and outputs based on the structure of the graph on creation.
    /// This way, we can reuse the same evaluator for multiple evaluations of the same graph
    /// without needing to reallocate memory each time.
    ///
    /// # Arguments
    /// * graph - The [Graph] to reduce.
    #[inline]
    pub fn new<N>(graph: &'a N) -> GraphEvaluator<'a, T, V>
    where
        N: AsRef<[GraphNode<T>]>,
    {
        let nodes = graph.as_ref();

        let mut total_inputs = 0;
        let mut input_ranges = Vec::with_capacity(nodes.len());

        for node in nodes {
            let k = node.incoming().len();
            input_ranges.push(total_inputs..total_inputs + k);
            total_inputs += k;
        }

        let mut output_indices: SmallVec<[usize; 8]> = SmallVec::new();
        for (i, n) in nodes.iter().enumerate() {
            if n.node_type() == NodeType::Output {
                output_indices.push(i);
            }
        }

        GraphEvaluator {
            nodes,
            inner: GraphEvalCache {
                inputs: vec![V::default(); total_inputs],
                outputs: vec![V::default(); nodes.len()],
                eval_order: nodes.iter_topological().map(|n| n.index()).collect(),
                input_ranges,
                output_indices,
            },
        }
    }

    pub fn take_cache(self) -> GraphEvalCache<V> {
        self.inner
    }
}

impl<T, V> EvalMut<[V], Vec<V>> for GraphEvaluator<'_, T, V>
where
    T: Eval<[V], V>,
    V: Clone + Default,
{
    #[inline]
    fn eval_mut(&mut self, input: &[V]) -> Vec<V> {
        let out_len = self.inner.output_indices.len();
        let mut buffer: Vec<V> = vec![V::default(); out_len];
        self.eval_into_mut(input, &mut buffer[..]);
        buffer
    }
}

impl<T, V> EvalIntoMut<[V], [V]> for GraphEvaluator<'_, T, V>
where
    T: Eval<[V], V>,
    V: Clone + Default,
{
    #[inline]
    fn eval_into_mut(&mut self, input: &[V], buffer: &mut [V]) {
        for &index in self.inner.eval_order.iter() {
            let node = &self.nodes[index];
            let incoming = node.incoming();

            if incoming.is_empty() {
                self.inner.outputs[index] = node.eval(input);
            } else {
                let range = &self.inner.input_ranges[index];
                let buf = &mut self.inner.inputs[range.clone()];

                for (dst, &src_idx) in buf.iter_mut().zip(incoming.iter()) {
                    *dst = self.inner.outputs[src_idx].clone();
                }

                self.inner.outputs[index] = node.eval(buf);
            }
        }

        let mut count = 0;
        for &idx in &self.inner.output_indices {
            buffer[count] = self.inner.outputs[idx].clone();
            count += 1;
        }
    }
}

impl<T, V> Eval<[Vec<V>], Vec<Vec<V>>> for Graph<T>
where
    T: Eval<[V], V>,
    V: Clone + Default,
{
    /// Evaluates the [Graph] with the given input `Vec<Vec<T>>`. Returns the output of the [Graph] as `Vec<Vec<T>>`.
    /// This is intended to be used when evaluating a batch of inputs.
    ///
    /// # Arguments
    /// * `input` - A `Vec<Vec<T>>` to evaluate the [Graph] with.
    ///
    /// # Returns
    /// * A `Vec<Vec<T>>` which is the output of the [Graph].
    #[inline]
    fn eval(&self, input: &[Vec<V>]) -> Vec<Vec<V>> {
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
    /// Evaluates the [GraphNode] with the given input. Returns the output of the [GraphNode].
    /// # Arguments
    /// * `inputs` - A `Vec` of `V` to evaluate the [GraphNode] with.
    ///
    /// # Returns
    /// * A `V` which is the output of the [GraphNode].
    #[inline]
    fn eval(&self, inputs: &[V]) -> V {
        self.value().eval(inputs)
    }
}

impl<'a, G, T, V> From<(&'a G, GraphEvalCache<V>)> for GraphEvaluator<'a, T, V>
where
    G: AsRef<[GraphNode<T>]>,
    T: Eval<[V], V>,
    V: Default + Clone,
{
    fn from((graph, cache): (&'a G, GraphEvalCache<V>)) -> Self {
        if cache.eval_order.is_empty() || graph.as_ref().len() != cache.eval_order.len() {
            return GraphEvaluator::new(graph);
        }

        GraphEvaluator {
            nodes: graph.as_ref(),
            inner: cache,
        }
    }
}

impl<'a, T, V> From<&'a Graph<T>> for GraphEvaluator<'a, T, V>
where
    T: Eval<[V], V>,
    V: Default + Clone,
{
    fn from(graph: &'a Graph<T>) -> Self {
        GraphEvaluator::new(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Graph, Op};

    fn round(value: f32, places: u32) -> f32 {
        let factor = 10_f32.powi(places as i32);
        (value * factor).round() / factor
    }

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

        let six = graph.eval(&[vec![1_f32]]);
        let seven = graph.eval(&[vec![2_f32]]);
        let eight = graph.eval(&[vec![3_f32]]);

        assert_eq!(six, vec![vec![6_f32]]);
        assert_eq!(seven, vec![vec![7_f32]]);
        assert_eq!(eight, vec![vec![8_f32]]);
        assert_eq!(graph.len(), 4);
    }

    #[test]
    fn test_graph_eval_recurrent() {
        let mut graph = Graph::<Op<f32>>::default();

        graph.insert(NodeType::Input, Op::var(0));
        graph.insert(NodeType::Vertex, Op::diff());
        graph.insert(NodeType::Output, Op::sigmoid());
        graph.insert(NodeType::Edge, Op::weight_with(-1.41));
        graph.insert(NodeType::Vertex, Op::sigmoid());
        graph.insert(NodeType::Vertex, Op::exp());
        graph.insert(NodeType::Edge, Op::weight_with(-1.10));
        graph.insert(NodeType::Vertex, Op::exp());
        graph.insert(NodeType::Vertex, Op::exp());
        graph.insert(NodeType::Vertex, Op::div());

        graph.attach(0, 1);
        graph.attach(1, 1);
        graph.attach(4, 1);
        graph.attach(7, 1);
        graph.attach(8, 1);
        graph.attach(1, 2);
        graph.attach(3, 2);
        graph.attach(6, 2);
        graph.attach(5, 3);
        graph.attach(1, 4);
        graph.attach(0, 5);
        graph.attach(9, 6);
        graph.attach(4, 7);
        graph.attach(7, 8);
        graph.attach(0, 9);
        graph.attach(9, 9);

        graph.set_cycles(vec![]);

        let mut evaluator = GraphEvaluator::new(&graph);

        let out1 = evaluator.eval_mut(&vec![0.0])[0];
        let out2 = evaluator.eval_mut(&vec![0.0])[0];
        let out3 = evaluator.eval_mut(&vec![0.0])[0];
        let out4 = evaluator.eval_mut(&vec![1.0])[0];
        let out5 = evaluator.eval_mut(&vec![0.0])[0];
        let out6 = evaluator.eval_mut(&vec![0.0])[0];
        let out7 = evaluator.eval_mut(&vec![0.0])[0];

        assert_eq!(round(out1, 3), 0.196);
        assert_eq!(round(out2, 3), 0.000);
        assert_eq!(round(out3, 3), 0.902);
        assert_eq!(round(out4, 3), 0.000);
        assert_eq!(round(out5, 3), 0.000);
        assert_eq!(round(out6, 3), 0.000);
        assert_eq!(round(out7, 3), 1.000);
    }
}
