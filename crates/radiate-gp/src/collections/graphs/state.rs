use crate::{
    Eval, EvalIntoMut, EvalMut, Graph, GraphEvaluator, GraphIterator, graphs::GraphEvalCache,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StatefulGraph<T, V> {
    inner: Graph<T>,
    state: Option<GraphEvalCache<V>>,
}

impl<T, V> StatefulGraph<T, V> {
    pub fn new(inner: Graph<T>) -> Self {
        StatefulGraph { inner, state: None }
    }

    pub fn reset(&mut self) {
        self.state = None;
    }

    pub fn input_dim(&self) -> usize {
        self.inner
            .get_nodes_of_type(crate::NodeType::Input)
            .collect::<Vec<_>>()
            .len()
    }

    pub fn output_dim(&self) -> usize {
        self.inner
            .get_nodes_of_type(crate::NodeType::Output)
            .collect::<Vec<_>>()
            .len()
    }

    pub fn eval_scoped<F, O>(&mut self, eval_fn: F) -> O
    where
        F: FnOnce(&mut Self) -> O,
        V: Copy + Default,
    {
        let current_state = self.state.take();
        let output = eval_fn(self);
        self.state = current_state;
        output
    }
}

impl<T, V> EvalIntoMut<[V], [V]> for StatefulGraph<T, V>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    fn eval_into_mut(&mut self, input: &[V], output: &mut [V]) {
        let mut evaluator = match self.state.take() {
            Some(c) => GraphEvaluator::from((&self.inner, c)),
            None => GraphEvaluator::new(&self.inner),
        };

        evaluator.eval_into_mut(input, output);
        self.state = Some(evaluator.take_cache());
    }
}

impl<T, V> EvalMut<[V], Vec<V>> for StatefulGraph<T, V>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    fn eval_mut(&mut self, input: &[V]) -> Vec<V> {
        let mut evaluator = match self.state.take() {
            Some(c) => GraphEvaluator::from((&self.inner, c)),
            None => GraphEvaluator::new(&self.inner),
        };

        let result = evaluator.eval_mut(input);
        self.state = Some(evaluator.take_cache());
        result
    }
}

impl<T, V> AsRef<Graph<T>> for StatefulGraph<T, V> {
    fn as_ref(&self) -> &Graph<T> {
        &self.inner
    }
}

impl<T, V> From<Graph<T>> for StatefulGraph<T, V>
where
    T: Eval<[V], V>,
{
    fn from(inner: Graph<T>) -> Self {
        StatefulGraph { inner, state: None }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_stateful_graph() {}
}
