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

    /// A `Fn(T) -> S` closure.
    pub struct Single<S>(PhantomData<S>);
    /// A `Fn(&[T]) -> Vec<S>` closure.
    pub struct Batch<S>(PhantomData<S>);
    /// A `Fn(&Genotype<C>) -> S` closure.
    pub struct Raw<S>(PhantomData<S>);
    /// A `Fn(&[&Genotype<C>]) -> Vec<S>` closure.
    pub struct RawBatch<S>(PhantomData<S>);
    /// A type implementing `FitnessFunction<T, S>`.
    pub struct CustomSingle<S>(PhantomData<S>);
    /// A type implementing `BatchFitnessFunction<T, S>`.
    pub struct CustomBatch<S>(PhantomData<S>);
    /// A type implementing `for<'a> FitnessFunction<&'a Genotype<C>, S>`.
    pub struct CustomRawSingle<S>(PhantomData<S>);
    /// A type implementing `for<'a> BatchFitnessFunction<&'a Genotype<C>, S>`.
    pub struct CustomRawBatch<S>(PhantomData<S>);
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
            self.evaluate(&ts).into_iter().map(Into::into).collect()
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
            self.evaluate(gs.as_slice())
                .into_iter()
                .map(Into::into)
                .collect()
        }))
    }
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
    fn evaluate(&self, individuals: &[T]) -> Vec<S>;
}

// /// Blanket implement FitnessFunction for any function that takes a single argument.
// /// This covers the base case for any function supplied to an engine that takes a decoded phenotype.
// impl<T, S, F> FitnessFunction<T, S> for F
// where
//     F: Fn(T) -> S + Send + Sync,
//     S: Into<Score>,
// {
//     fn evaluate(&self, individual: T) -> S {
//         self(individual)
//     }
// }

// /// Blanket implement BatchFitnessFunction for any function that takes a slice of arguments.
// /// This covers the base case for any function supplied to an engine that takes a batch of decoded phenotypes.
// impl<T, S, F> BatchFitnessFunction<T, S> for F
// where
//     F: for<'a> Fn(&'a [T]) -> Vec<S> + Send + Sync,
//     S: Into<Score>,
// {
//     fn evaluate(&self, individuals: Vec<T>) -> Vec<S> {
//         self(&individuals)
//     }
// }

// impl<T, S, F> FitnessFunction<T, S> for SingleFn<F>
// where
//     F: Fn(T) -> S + Send + Sync,
//     S: Into<Score>,
// {
//     fn evaluate(&self, individual: T) -> S {
//         self.0(individual)
//     }
// }

// impl<T, S, F> BatchFitnessFunction<T, S> for SingleFn<F>
// where
//     F: Fn(T) -> S + Send + Sync,
//     S: Into<Score>,
// {
//     fn evaluate(&self, individuals: Vec<T>) -> Vec<S> {
//         individuals.into_iter().map(|t| self.0(t)).collect()
//     }
// }

// impl<T, S, F> BatchFitnessFunction<T, S> for BatchedFn<F>
// where
//     F: Fn(&[T]) -> Vec<S> + Send + Sync,
//     S: Into<Score>,
// {
//     fn evaluate(&self, individuals: Vec<T>) -> Vec<S> {
//         self.0(&individuals)
//     }
// }

// impl<T, S, F> FitnessFunction<T, S> for BatchedFn<F>
// where
//     F: Fn(&[T]) -> Vec<S> + Send + Sync,
//     S: Into<Score>,
// {
//     fn evaluate(&self, individual: T) -> S {
//         self.0(std::slice::from_ref(&individual))
//             .into_iter()
//             .next()
//             .expect("BatchedFn returned an empty Vec for a single individual")
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::sync::{
//         Arc,
//         atomic::{AtomicUsize, Ordering},
//     };

//     #[test]
//     fn single_fn_serves_single_eval() {
//         let f: SingleFn<_> = SingleFn(|x: i32| x as f32 * 2.0);
//         let score: f32 = FitnessFunction::evaluate(&f, 5);
//         assert_eq!(score, 10.0);
//     }

//     #[test]
//     fn single_fn_serves_batch_eval_via_fan_out() {
//         let f: SingleFn<_> = SingleFn(|x: i32| x as f32 * 2.0);
//         let scores: Vec<f32> = BatchFitnessFunction::evaluate(&f, vec![1, 2, 3, 4]);
//         assert_eq!(scores, vec![2.0, 4.0, 6.0, 8.0]);
//     }

//     #[test]
//     fn single_fn_batch_calls_inner_once_per_individual() {
//         let calls = Arc::new(AtomicUsize::new(0));
//         let f = {
//             let calls = Arc::clone(&calls);
//             SingleFn(move |x: i32| {
//                 calls.fetch_add(1, Ordering::Relaxed);
//                 x as f32
//             })
//         };
//         let _: Vec<f32> = BatchFitnessFunction::evaluate(&f, vec![10, 20, 30]);
//         assert_eq!(calls.load(Ordering::Relaxed), 3);
//     }

//     #[test]
//     fn batched_fn_serves_batch_eval_directly() {
//         let f = BatchedFn(|xs: &[i32]| xs.iter().map(|&x| x as f32 * 0.5).collect());
//         let scores: Vec<f32> = BatchFitnessFunction::evaluate(&f, vec![2, 4, 6, 8]);
//         assert_eq!(scores, vec![1.0, 2.0, 3.0, 4.0]);
//     }

//     #[test]
//     fn batched_fn_serves_single_eval_via_slice_of_one() {
//         let f = BatchedFn(|xs: &[i32]| xs.iter().map(|&x| x as f32 * 0.5).collect());
//         let score: f32 = FitnessFunction::evaluate(&f, 6);
//         assert_eq!(score, 3.0);
//     }

//     #[test]
//     fn batched_fn_single_eval_calls_inner_once_with_one_element() {
//         let calls = Arc::new(AtomicUsize::new(0));
//         let last_len = Arc::new(AtomicUsize::new(0));
//         let f = {
//             let calls = Arc::clone(&calls);
//             let last_len = Arc::clone(&last_len);
//             BatchedFn(move |xs: &[i32]| {
//                 calls.fetch_add(1, Ordering::Relaxed);
//                 last_len.store(xs.len(), Ordering::Relaxed);
//                 xs.iter().map(|&x| x as f32).collect()
//             })
//         };

//         let _: f32 = FitnessFunction::evaluate(&f, 42);
//         assert_eq!(calls.load(Ordering::Relaxed), 1);
//         assert_eq!(last_len.load(Ordering::Relaxed), 1);

//         let _: Vec<f32> = BatchFitnessFunction::evaluate(&f, vec![1, 2, 3]);
//         assert_eq!(calls.load(Ordering::Relaxed), 2);
//         assert_eq!(last_len.load(Ordering::Relaxed), 3);
//     }

//     // --- Composite-shape integration ---
//     //
//     // CompositeFitnessFn stores `Arc<dyn for<'a> FitnessFunction<&'a T, S>>` —
//     // the &T variant. Verify both wrappers can be coerced to that dyn shape
//     // when their inner closure is the &T form.

//     #[test]
//     fn single_fn_with_ref_input_works_in_composite_shape() {
//         let f = SingleFn(|x: &i32| (*x as f32) + 1.0);
//         let dynamic: Arc<dyn for<'a> FitnessFunction<&'a i32, f32>> = Arc::new(f);
//         let val = 7;
//         assert_eq!(dynamic.evaluate(&val), 8.0);
//     }

//     #[test]
//     fn batched_fn_with_ref_slice_input_works_in_composite_shape() {
//         let f = BatchedFn(|xs: &[&i32]| xs.iter().map(|x| **x as f32 + 1.0).collect());
//         let dynamic: Arc<dyn for<'a> FitnessFunction<&'a i32, f32>> = Arc::new(f);
//         let val = 7;
//         assert_eq!(dynamic.evaluate(&val), 8.0);
//     }
// }
