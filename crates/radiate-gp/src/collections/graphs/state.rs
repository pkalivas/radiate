use crate::{Eval, EvalIntoMut, EvalMut, Graph, GraphEvaluator, graphs::GraphEvalCache};
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
}

impl<T, V> From<Graph<T>> for StatefulGraph<T, V>
where
    T: Eval<[V], V>,
{
    fn from(inner: Graph<T>) -> Self {
        StatefulGraph { inner, state: None }
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

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_stateful_graph() {}
}
