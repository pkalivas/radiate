use super::marker;
use crate::FitnessFunction;
use std::sync::Arc;

const MIN_SCORE: f32 = 1e-8;

type ObjectiveFn<T> = Arc<dyn Fn(&T) -> f32 + Send + Sync>;

/// Converts a borrowed-input objective into the form [CompositeFitnessFn] stores. Closures
/// (`Fn(&T) -> f32`) and types implementing `for<'a> FitnessFunction<&'a T, f32>` (such as
/// [NoveltySearch](super::NoveltySearch)) are both accepted; `M` is the same marker used by
/// [IntoFitness](super::IntoFitness) and is always inferred.
pub trait IntoObjectiveFn<T, M> {
    fn into_objective_fn(self) -> ObjectiveFn<T>;
}

impl<T, F> IntoObjectiveFn<T, marker::Single<f32>> for F
where
    F: Fn(&T) -> f32 + Send + Sync + 'static,
{
    fn into_objective_fn(self) -> ObjectiveFn<T> {
        Arc::new(self)
    }
}

impl<T, F> IntoObjectiveFn<T, marker::CustomSingle<f32>> for F
where
    F: for<'a> FitnessFunction<&'a T, f32> + 'static,
{
    fn into_objective_fn(self) -> ObjectiveFn<T> {
        Arc::new(move |individual: &T| self.evaluate(individual))
    }
}

/// A composite fitness function that combines multiple fitness objectives with weights.
///
/// This struct allows you to create a single fitness function from multiple sub-objectives,
/// each with its own weight. The final fitness score is calculated as a weighted average
/// of all individual objective scores.
///
/// # Generic Parameters
/// - `T`: The type of individual being evaluated
///
/// # Examples
///
/// ```rust
/// use radiate_core::fitness::{CompositeFitnessFn, FitnessFunction};
///
/// fn accuracy_fn(individual: &Vec<f32>) -> f32 {
///     individual.iter().cloned().fold(0.0, |acc, x| acc + x) / individual.len() as f32
/// }
///
/// fn complexity_fn(individual: &Vec<f32>) -> f32 {
///     individual.iter().cloned().fold(0.0, |acc, x| acc + x.powi(2)) / individual.len() as f32
/// }
///
/// // Create a composite fitness function
/// let composite = CompositeFitnessFn::new()
///     .add_weighted_fn(accuracy_fn, 0.7)      // 70% weight on accuracy
///     .add_weighted_fn(complexity_fn, 0.3);   // 30% weight on complexity
///
/// // Evaluate an individual
/// let fitness = composite.evaluate(vec![0.5, 0.5]);
/// ```
pub struct CompositeFitnessFn<T> {
    objectives: Vec<ObjectiveFn<T>>,
    weights: Vec<f32>,
}

impl<T> CompositeFitnessFn<T> {
    pub fn new() -> Self {
        Self {
            objectives: Vec::new(),
            weights: Vec::new(),
        }
    }

    pub fn add_weighted_fn<M>(
        mut self,
        fitness_fn: impl IntoObjectiveFn<T, M>,
        weight: f32,
    ) -> Self {
        self.objectives.push(fitness_fn.into_objective_fn());
        self.weights.push(weight);
        self
    }

    pub fn add_fitness_fn<M>(mut self, fitness_fn: impl IntoObjectiveFn<T, M>) -> Self {
        self.objectives.push(fitness_fn.into_objective_fn());
        self.weights.push(1.0);
        self
    }

    fn weighted_score(&self, individual: &T) -> f32 {
        let mut total_score = 0.0;
        let mut total_weight = 0.0;
        for (objective, weight) in self.objectives.iter().zip(&self.weights) {
            total_score += objective(individual) * weight;
            total_weight += weight;
        }

        total_score / total_weight.max(MIN_SCORE)
    }
}

impl<T> Default for CompositeFitnessFn<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of `FitnessFunction` for `CompositeFitnessFn` when `S = f32`. This is
/// assumed to be the base case for most fitness functions and for now, as I can see it,
/// is the best way to apply the weight to the different fitness functions.
///
/// This implementation calculates the weighted average of all objective scores.
/// The final fitness score is computed as:
///
/// ``` text
/// final_score = Σ(score_i × weight_i) / Σ(weight_i)
/// ```
impl<T: Send + Sync> FitnessFunction<T> for CompositeFitnessFn<T> {
    fn evaluate(&self, individual: T) -> f32 {
        self.weighted_score(&individual)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fitness::FitnessFunction;

    // Mock fitness functions for testing
    fn mock_accuracy_fn(individual: &i32) -> f32 {
        *individual as f32 * 0.1
    }

    fn mock_complexity_fn(individual: &i32) -> f32 {
        -*individual as f32 * 0.05
    }

    #[test]
    fn test_add_weighted_fn() {
        let composite = CompositeFitnessFn::new()
            .add_weighted_fn(mock_accuracy_fn, 0.7)
            .add_weighted_fn(mock_complexity_fn, 0.3);

        assert_eq!(composite.objectives.len(), 2);
        assert_eq!(composite.weights, vec![0.7, 0.3]);
    }

    #[test]
    fn test_add_fitness_fn() {
        let composite = CompositeFitnessFn::new()
            .add_fitness_fn(mock_accuracy_fn)
            .add_fitness_fn(mock_complexity_fn);

        assert_eq!(composite.objectives.len(), 2);
        assert_eq!(composite.weights, vec![1.0, 1.0]);
    }

    #[test]
    fn test_evaluate_single() {
        let composite = CompositeFitnessFn::new()
            .add_weighted_fn(mock_accuracy_fn, 0.7)
            .add_weighted_fn(mock_complexity_fn, 0.3);

        let individual = 10;
        let fitness = FitnessFunction::evaluate(&composite, individual);

        // Expected: (10 * 0.1 * 0.7 + (-10 * 0.05) * 0.3) / (0.7 + 0.3)
        // = (0.7 - 0.15) / 1.0 = 0.55
        assert!((fitness - 0.55).abs() < 1e-6);
    }

    #[test]
    fn test_empty_composite() {
        let composite = CompositeFitnessFn::<i32>::new();
        let individual = 10;
        let fitness = FitnessFunction::evaluate(&composite, individual);

        // Should return 0.0 when no objectives are defined
        assert_eq!(fitness, 0.0);
    }
}
