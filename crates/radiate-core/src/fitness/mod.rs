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

impl<T, S, F> FitnessFunction<T, S> for F
where
    F: Fn(T) -> S + Send + Sync,
    S: Into<Score>,
{
    fn evaluate(&self, individual: T) -> S {
        self(individual)
    }
}
