mod composite;
mod novelty;

use crate::Score;
pub use composite::CompositeFitnessFn;
pub use novelty::{Novelty, NoveltySearch};

pub trait FitnessFunction<T, S = f32>: Send + Sync
where
    S: Into<Score>,
{
    fn evaluate(&self, individual: T) -> S;
}

/// Fitness function for evaluating a batch of individuals
/// Its important to note that the indices of the individuals in the input slice
/// must match the indices of the corresponding fitness values in the output vector.
pub trait BatchFitnessFunction<T, S = f32>: Send + Sync
where
    S: Into<Score>,
{
    fn evaluate(&self, individuals: Vec<T>) -> Vec<S>;
}

/// Blanket implement FitnessFunction for any function that takes a single argument.
/// This covers the base case for any function supplied to an engine that takes a decoded phenotype.
impl<T, S, F> FitnessFunction<T, S> for F
where
    F: Fn(T) -> S + Send + Sync,
    S: Into<Score>,
{
    fn evaluate(&self, individual: T) -> S {
        self(individual)
    }
}

/// Blanket implement BatchFitnessFunction for any function that takes a slice of arguments.
/// This covers the base case for any function supplied to an engine that takes a batch of decoded phenotypes.
impl<T, S, F> BatchFitnessFunction<T, S> for F
where
    F: for<'a> Fn(&'a [T]) -> Vec<S> + Send + Sync,
    S: Into<Score>,
{
    fn evaluate(&self, individuals: Vec<T>) -> Vec<S> {
        self(&individuals)
    }
}
