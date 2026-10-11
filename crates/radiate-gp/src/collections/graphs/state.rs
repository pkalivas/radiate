use crate::{
    Eval, EvalMut, Graph, GraphIterator,
    graphs::{GraphEvalCache, eval},
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
    {
        let current_state = self.state.take();
        let output = eval_fn(self);
        self.state = current_state;
        output
    }
}

impl<T, V> StatefulGraph<T, V>
where
    V: Default + Clone,
{
    /// Resets every node's state to default, as if no sample had been evaluated yet.
    pub fn reset(&mut self) {
        if let Some(cache) = &mut self.state {
            cache.reset_state();
        }
    }
}

impl<T, V> EvalMut<[V], Vec<V>> for StatefulGraph<T, V>
where
    T: Eval<[V], V>,
    V: Copy + Default,
{
    fn eval_mut(&mut self, input: &[V]) -> Vec<V> {
        let cache = self
            .state
            .get_or_insert_with(|| GraphEvalCache::new(self.inner.as_ref()));

        let mut outputs = vec![V::default(); cache.plan().output_indices().len()];
        eval::process_single_row_into(cache, self.inner.as_ref(), input, &mut outputs);
        outputs
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
    V: Default + Clone,
{
    fn from(inner: Graph<T>) -> Self {
        Self::new(inner)
    }
}
