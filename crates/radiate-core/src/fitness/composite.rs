use crate::{FitnessFunction, Score};
use std::sync::Arc;

pub struct CompositeFitnessFn<T, S> {
    objectives: Vec<Arc<dyn for<'a> FitnessFunction<&'a T, S>>>,
    weights: Vec<f32>,
}

impl<T, S> CompositeFitnessFn<T, S>
where
    S: Into<Score> + Clone,
{
    pub fn new() -> Self {
        Self {
            objectives: Vec::new(),
            weights: Vec::new(),
        }
    }

    pub fn add_weighted_fn(
        mut self,
        fitness_fn: impl for<'a> FitnessFunction<&'a T, S> + 'static,
        weight: f32,
    ) -> Self
    where
        S: Into<Score>,
    {
        self.objectives.push(Arc::new(fitness_fn));
        self.weights.push(weight);
        self
    }

    pub fn add_fitness_fn(
        mut self,
        fitness_fn: impl for<'a> FitnessFunction<&'a T, S> + 'static,
    ) -> Self
    where
        S: Into<Score>,
    {
        self.objectives.push(Arc::new(fitness_fn));
        self.weights.push(1.0);
        self
    }
}

impl<T> FitnessFunction<T> for CompositeFitnessFn<T, f32> {
    fn evaluate(&self, individual: T) -> f32 {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        for (objective, weight) in self.objectives.iter().zip(&self.weights) {
            let score = objective.evaluate(&individual);
            total_score += score * weight;
            total_weight += weight;
        }

        total_score / total_weight.max(1e-8)
    }
}
