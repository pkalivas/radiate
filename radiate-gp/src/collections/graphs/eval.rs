use super::{iter::GraphIterator, GraphNode};
use crate::{Eval, EvalMut, NodeCell, NodeType};

/// `GraphReducer` is a struct that is used to evaluate a `Graph` of `Node`s. It uses the `GraphIterator`
/// to traverse the `Graph` in a sudo-topological order and evaluate the nodes in the correct order.
///
/// On the first iteration it caches the order of nodes in the `Graph` and then uses that order to
/// evaluate the nodes in the correct order. This is a massive performance improvement.
pub struct GraphEvaluator<'a, C, T>
where
    C: NodeCell,
{
    nodes: &'a [GraphNode<C>],
    output_size: usize,
    eval_order: Vec<usize>,
    outputs: Vec<T>,
    inputs: Vec<Vec<T>>,
}

impl<'a, C, T> GraphEvaluator<'a, C, T>
where
    C: NodeCell,
    T: Default + Clone,
{
    /// Creates a new `GraphReducer` with the given `Graph`. Will cache the order of nodes in
    /// the `Graph` on the first iteration. On initialization the `GraphReducer` will cache the
    /// output size of the `Graph` to be used in the `reduce` method and create a vec of `Tracer`
    /// which will be used to evaluate the `Graph` in the `reduce` method.
    ///
    /// # Arguments
    /// * `graph` - The `Graph` to reduce.
    pub fn new<N>(graph: &'a N) -> GraphEvaluator<'a, C, T>
    where
        N: AsRef<[GraphNode<C>]>,
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

/// Implements the `Reduce` trait for `GraphReducer`.
impl<'a, C, T> EvalMut<[T], Vec<T>> for GraphEvaluator<'a, C, T>
where
    C: NodeCell + Eval<[T], T>,
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

#[cfg(test)]
mod tests {
    use radiate::random_provider;

    use crate::graphs::builder::GraphBuilder;

    use super::*;

    #[test]
    fn test_graph_eval_simple() {
        random_provider::set_seed(2);

        let builder = GraphBuilder::regression(3);

        let graph = builder.lstm(3, 3, 3);

        for node in graph.iter() {
            println!("{:?}", node);
        }

        let mut evaluator = GraphEvaluator::new(&graph);

        let input = vec![1.0, 2.0, 3.0];
        let output = evaluator.eval_mut(&input);

        println!("{:?}", output);

        assert_eq!(output.len(), 3);
        assert_eq!(output[0], 0.33010882);
        assert_eq!(output[1], 0.0);
        assert_eq!(output[2], 0.0);
    }
}
