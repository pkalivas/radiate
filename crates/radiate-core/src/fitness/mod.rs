mod composite;
mod novelty;

use std::sync::Arc;

use crate::{Chromosome, Genotype, Score, problem::FitnessKind};
pub use composite::{CompositeFitnessFn, IntoObjectiveFn};
pub use novelty::{Novelty, NoveltySearch};

/// Zero-sized types that tell the [IntoFitness] impls apart. Each impl is keyed by one of
/// these, so the compiler can pick the impl that matches a fitness function's shape. They are
/// always inferred and never constructed.
#[doc(hidden)]
pub mod marker {
    use std::marker::PhantomData;
    pub struct Single<S>(PhantomData<S>);
    pub struct Batch<S>(PhantomData<S>);
    pub struct Raw<S>(PhantomData<S>);
    pub struct RawBatch<S>(PhantomData<S>);
    pub struct CustomSingle<S>(PhantomData<S>);
    pub struct CustomBatch<S>(PhantomData<S>);
    pub struct CustomRawSingle<S>(PhantomData<S>);
    pub struct CustomRawBatch<S>(PhantomData<S>);
}

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

pub trait IntoFitness<C: Chromosome, T, M> {
    fn into_fitness(self) -> FitnessKind<C, T>;
}

impl<C, T, S, F> IntoFitness<C, T, marker::Single<S>> for F
where
    C: Chromosome,
    F: Fn(T) -> S + Send + Sync + 'static,
    S: Into<Score>,
{
    fn into_fitness(self) -> FitnessKind<C, T> {
        FitnessKind::Single(Arc::new(move |t| self(t).into()))
    }
}

impl<C, T, S, F> IntoFitness<C, T, marker::Batch<S>> for F
where
    C: Chromosome,
    F: Fn(&[T]) -> Vec<S> + Send + Sync + 'static,
    S: Into<Score>,
{
    fn into_fitness(self) -> FitnessKind<C, T> {
        FitnessKind::Batch(Arc::new(move |ts| {
            self(&ts).into_iter().map(Into::into).collect()
        }))
    }
}

impl<C, T, S, F> IntoFitness<C, T, marker::Raw<S>> for F
where
    C: Chromosome,
    F: Fn(&Genotype<C>) -> S + Send + Sync + 'static,
    S: Into<Score>,
{
    fn into_fitness(self) -> FitnessKind<C, T> {
        FitnessKind::RawSingle(Arc::new(move |g| self(g).into()))
    }
}

impl<C, T, S, F> IntoFitness<C, T, marker::RawBatch<S>> for F
where
    C: Chromosome,
    F: Fn(&[&Genotype<C>]) -> Vec<S> + Send + Sync + 'static,
    S: Into<Score>,
{
    fn into_fitness(self) -> FitnessKind<C, T> {
        FitnessKind::RawBatch(Arc::new(move |gs| {
            self(gs.as_slice()).into_iter().map(Into::into).collect()
        }))
    }
}

impl<C, T, S, F> IntoFitness<C, T, marker::CustomSingle<S>> for F
where
    C: Chromosome,
    F: FitnessFunction<T, S> + Send + Sync + 'static,
    S: Into<Score>,
{
    fn into_fitness(self) -> FitnessKind<C, T> {
        FitnessKind::Single(Arc::new(move |t| self.evaluate(t).into()))
    }
}

impl<C, T, S, F> IntoFitness<C, T, marker::CustomBatch<S>> for F
where
    C: Chromosome,
    F: BatchFitnessFunction<T, S> + Send + Sync + 'static,
    S: Into<Score>,
{
    fn into_fitness(self) -> FitnessKind<C, T> {
        FitnessKind::Batch(Arc::new(move |ts| {
            self.evaluate(ts).into_iter().map(Into::into).collect()
        }))
    }
}

impl<C, T, S, F> IntoFitness<C, T, marker::CustomRawSingle<S>> for F
where
    C: Chromosome,
    F: for<'a> FitnessFunction<&'a Genotype<C>, S> + Send + Sync + 'static,
    S: Into<Score>,
{
    fn into_fitness(self) -> FitnessKind<C, T> {
        FitnessKind::RawSingle(Arc::new(move |g| self.evaluate(g).into()))
    }
}

impl<C, T, S, F> IntoFitness<C, T, marker::CustomRawBatch<S>> for F
where
    C: Chromosome,
    F: for<'a> BatchFitnessFunction<&'a Genotype<C>, S> + Send + Sync + 'static,
    S: Into<Score>,
{
    fn into_fitness(self) -> FitnessKind<C, T> {
        FitnessKind::RawBatch(Arc::new(move |gs| {
            self.evaluate(gs).into_iter().map(Into::into).collect()
        }))
    }
}
