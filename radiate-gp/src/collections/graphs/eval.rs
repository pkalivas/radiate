use super::{iter::GraphIterator, Graph, GraphNode};
use crate::{node::Node, Eval, EvalMut, NodeType};

/// `GraphReducer` is a struct that is used to evaluate a `Graph` of `Node`s. It uses the `GraphIterator`
/// to traverse the `Graph` in a sudo-topological order and evaluate the nodes in the correct order.
///
/// On the first iteration it caches the order of nodes in the `Graph` and then uses that order to
/// evaluate the nodes in the correct order. This is a massive performance improvement.
pub struct GraphEvaluator<'a, T, V> {
    nodes: &'a [GraphNode<T>],
    output_size: usize,
    eval_order: Vec<usize>,
    outputs: Vec<V>,
    inputs: Vec<Vec<V>>,
}

impl<'a, T, V> GraphEvaluator<'a, T, V>
where
    T: Eval<[V], V>,
    V: Default + Clone,
{
    /// Creates a new `GraphEvaluator` with the given `Graph`. Will cache the order of nodes in
    /// the `Graph` on the first iteration. On initialization the `GraphEvaluator` will cache the
    /// output size of the `Graph` to be used in the `reduce` method and create a vec of `Tracer`
    /// which will be used to evaluate the `Graph` in the `reduce` method.
    ///
    /// # Arguments
    /// * `graph` - The `Graph` to reduce.
    pub fn new<N>(graph: &'a N) -> GraphEvaluator<'a, T, V>
    where
        N: AsRef<[GraphNode<T>]>,
    {
        let nodes = graph.as_ref();

        GraphEvaluator {
            nodes,
            output_size: nodes
                .iter()
                .filter(|node| node.node_type() == NodeType::Output)
                .count(),
            inputs: nodes
                .iter()
                .map(|node| vec![V::default(); node.incoming().len()])
                .collect::<Vec<Vec<V>>>(),
            eval_order: nodes.iter_topological().map(|node| node.index()).collect(),
            outputs: vec![V::default(); nodes.len()],
        }
    }
}

/// Implements the `EvalMut` trait for `GraphEvaluator`.
impl<'a, T, V> EvalMut<[V], Vec<V>> for GraphEvaluator<'a, T, V>
where
    T: Eval<[V], V>,
    V: Clone + Default,
{
    /// Evaluates the `Graph` with the given input. Returns the output of the `Graph`.
    /// The `reduce` method will cache the order of nodes in the `Graph` on the first iteration.
    /// On subsequent iterations it will use the cached order to evaluate the `Graph` in the correct order.
    ///
    /// # Arguments
    /// * `input` - A `Vec` of `T` to evaluate the `Graph` with.
    ///
    ///  # Returns
    /// * A `Vec` of `T` which is the output of the `Graph`.
    #[inline]
    fn eval_mut(&mut self, input: &[V]) -> Vec<V> {
        let mut output = Vec::with_capacity(self.output_size);
        for index in self.eval_order.iter() {
            let node = &self.nodes[*index];
            if node.incoming().is_empty() {
                self.outputs[node.index()] = node.eval(input);
            } else {
                for (idx, incoming) in node.incoming().iter().enumerate() {
                    self.inputs[node.index()][idx] = self.outputs[*incoming].clone();
                }

                self.outputs[node.index()] = node.eval(&self.inputs[node.index()]);
            }

            if node.node_type() == NodeType::Output {
                output.push(self.outputs[node.index()].clone());
            }
        }

        output
    }
}

impl<T, V> Eval<[V], Vec<V>> for Graph<T>
where
    T: Eval<[V], V>,
    V: Clone + Default,
{
    /// Evaluates the `Graph` with the given input. Returns the output of the `Graph`.
    ///
    /// # Arguments
    /// * `input` - A `Vec` of `T` to evaluate the `Graph` with.
    ///
    ///  # Returns
    /// * A `Vec` of `T` which is the output of the `Graph`.
    #[inline]
    fn eval(&self, input: &[V]) -> Vec<V> {
        let mut evaluator = GraphEvaluator::new(self);
        evaluator.eval_mut(input)
    }
}

impl<T, V> Eval<Vec<Vec<V>>, Vec<Vec<V>>> for Graph<T>
where
    T: Eval<[V], V>,
    V: Clone + Default,
{
    /// Evaluates the `Graph` with the given input 'Vec<Vec<T>>'. Returns the output of the `Graph` as 'Vec<Vec<T>>'.
    /// This is inteded to be used when evaluating a batch of inputs.
    ///
    /// # Arguments
    /// * `input` - A `Vec<Vec<T>>` to evaluate the `Graph` with.
    ///
    /// # Returns
    /// * A `Vec<Vec<T>>` which is the output of the `Graph`.
    #[inline]
    fn eval(&self, input: &Vec<Vec<V>>) -> Vec<Vec<V>> {
        let mut output = Vec::with_capacity(self.len());
        let mut evaluator = GraphEvaluator::new(self);

        for inputs in input.iter() {
            output.push(evaluator.eval_mut(inputs));
        }

        output
    }
}

impl<T: Eval<[V], V>, V: Clone> Eval<[V], V> for GraphNode<T> {
    /// Evaluates the `GraphNode` with the given input. Returns the output of the `GraphNode`.
    /// # Arguments
    /// * `inputs` - A `Vec` of `T` to evaluate the `GraphNode` with.
    ///
    /// # Returns
    /// * A `T` which is the output of the `GraphNode`.
    #[inline]
    fn eval(&self, inputs: &[V]) -> V {
        self.value().eval(inputs)
    }
}
