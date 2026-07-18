use crate::{
    Eval, EvalInto, EvalIntoMut, EvalMut, Graph, GraphEvaluator, Op, graphs::GraphEvalCache,
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
}

// impl<T, V> From<Graph<T>> for StatefulGraph<T, V> {
//     fn from(inner: Graph<T>) -> Self {
//         StatefulGraph { inner, state: None }
//     }
// }

// impl<T, V> From<(Graph<T>, GraphEvalCache<V>)> for StatefulGraph<T, V> {
//     fn from((inner, state): (Graph<T>, GraphEvalCache<V>)) -> Self {
//         StatefulGraph {
//             inner,
//             state: Some(state),
//         }
//     }
// }

impl<T> From<Graph<Op<T>>> for StatefulGraph<Op<T>, T>
where
    T: Copy + Default,
{
    fn from(inner: Graph<Op<T>>) -> Self {
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

impl<T, V> EvalInto<[Vec<V>], Vec<Vec<V>>> for StatefulGraph<T, V>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    fn eval_into(&self, input: &[Vec<V>], buffer: &mut Vec<Vec<V>>) {
        let mut evaluator = match self.state.as_ref() {
            Some(c) => GraphEvaluator::from((&self.inner, c.clone())),
            None => GraphEvaluator::new(&self.inner),
        };

        for i in 0..input.len() {
            evaluator.eval_into_mut(&input[i], &mut buffer[i]);
        }
    }
}

impl<T, V> Eval<[V], Vec<V>> for StatefulGraph<T, V>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    fn eval(&self, input: &[V]) -> Vec<V> {
        let mut evaluator = match self.state.as_ref() {
            Some(c) => GraphEvaluator::from((&self.inner, c.clone())),
            None => GraphEvaluator::new(&self.inner),
        };

        evaluator.eval_mut(input)
    }
}
