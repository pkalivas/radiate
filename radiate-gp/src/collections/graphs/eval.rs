use super::{iter::GraphIterator, Graph, GraphNode};
use crate::{Eval, EvalMut, NodeType};

/// `GraphReducer` is a struct that is used to evaluate a `Graph` of `Node`s. It uses the `GraphIterator`
/// to traverse the `Graph` in a sudo-topological order and evaluate the nodes in the correct order.
///
/// On the first iteration it caches the order of nodes in the `Graph` and then uses that order to
/// evaluate the nodes in the correct order. This is a massive performance improvement.
pub struct GraphEvaluator<'a, T> {
    nodes: &'a [GraphNode<T>],
    output_size: usize,
    eval_order: Vec<usize>,
    outputs: Vec<T>,
    inputs: Vec<Vec<T>>,
}

impl<'a, T> GraphEvaluator<'a, T>
where
    T: Default + Clone,
{
    /// Creates a new `GraphEvaluator` with the given `Graph`. Will cache the order of nodes in
    /// the `Graph` on the first iteration. On initialization the `GraphEvaluator` will cache the
    /// output size of the `Graph` to be used in the `reduce` method and create a vec of `Tracer`
    /// which will be used to evaluate the `Graph` in the `reduce` method.
    ///
    /// # Arguments
    /// * `graph` - The `Graph` to reduce.
    pub fn new<N>(graph: &'a N) -> GraphEvaluator<'a, T>
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
                .map(|node| vec![T::default(); node.incoming().len()])
                .collect::<Vec<Vec<T>>>(),
            eval_order: nodes.iter_topological().map(|node| node.index()).collect(),
            outputs: vec![T::default(); nodes.len()],
        }
    }
}

/// Implements the `EvalMut` trait for `GraphEvaluator`.
impl<T> EvalMut<[T], Vec<T>> for GraphEvaluator<'_, T>
where
    T: Clone + Default,
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
    fn eval_mut(&mut self, input: &[T]) -> Vec<T> {
        let mut output = Vec::with_capacity(self.output_size);
        for index in self.eval_order.iter() {
            let node = &self.nodes[*index];
            if node.incoming().is_empty() {
                self.outputs[node.index()] = node.value().eval(input);
            } else {
                let mut count = 0;
                for incoming in node.incoming() {
                    self.inputs[node.index()][count] = self.outputs[*incoming].clone();
                    count += 1;
                }

                self.outputs[node.index()] = node.value().eval(&self.inputs[node.index()]);
            }

            if node.node_type() == NodeType::Output {
                output.push(self.outputs[node.index()].clone());
            }
        }

        output
    }
}

impl<T> Eval<[T], Vec<T>> for Graph<T>
where
    T: Clone + Default,
{
    /// Evaluates the `Graph` with the given input. Returns the output of the `Graph`.
    ///
    /// # Arguments
    /// * `input` - A `Vec` of `T` to evaluate the `Graph` with.
    ///
    ///  # Returns
    /// * A `Vec` of `T` which is the output of the `Graph`.
    #[inline]
    fn eval(&self, input: &[T]) -> Vec<T> {
        let mut evaluator = GraphEvaluator::new(self);
        evaluator.eval_mut(input)
    }
}

impl<T> Eval<Vec<Vec<T>>, Vec<Vec<T>>> for Graph<T>
where
    T: Clone + Default,
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
    fn eval(&self, input: &Vec<Vec<T>>) -> Vec<Vec<T>> {
        let mut output = Vec::with_capacity(self.len());
        let mut evaluator = GraphEvaluator::new(self);

        for inputs in input.iter() {
            output.push(evaluator.eval_mut(inputs));
        }

        output
    }
}

impl<T: Clone> Eval<[T], T> for GraphNode<T> {
    fn eval(&self, inputs: &[T]) -> T {
        self.value().eval(inputs)
    }
}
