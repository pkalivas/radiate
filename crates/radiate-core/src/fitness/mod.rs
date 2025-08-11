mod composite;
mod novelty;

pub use composite::CompositeFitnessFn;
pub use novelty::{FitnessDescriptor, Novelty, NoveltySearch};

use crate::Score;

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
    fn evaluate(&self, individuals: &[T]) -> Vec<S>;
}

impl<T, S, F> FitnessFunction<T, S> for F
where
    F: Fn(T) -> S + Send + Sync,
    S: Into<Score>,
{
    fn evaluate(&self, individual: T) -> S {
        self(individual)
    }
}

impl<T, S, F> BatchFitnessFunction<T, S> for F
where
    F: for<'a> Fn(&'a [T]) -> Vec<S> + Send + Sync,
    S: Into<Score>,
{
    fn evaluate(&self, individuals: &[T]) -> Vec<S> {
        self(individuals)
    }
}
