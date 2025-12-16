use crate::GeneticEngineBuilder;
use radiate_core::{
    Chromosome, Codec, Genotype, Problem, Score,
    fitness::{BatchFitnessFunction, FitnessFunction},
};
use std::sync::Arc;

type FitnessFn<T> = dyn Fn(T) -> Score + Send + Sync;
type BatchFitnessFn<T> = dyn Fn(Vec<T>) -> Vec<Score> + Send + Sync;
type RawFitnessFn<C> = dyn Fn(&Genotype<C>) -> Score + Send + Sync;
type RawBatchFitnessFn<C> = dyn Fn(Vec<&Genotype<C>>) -> Vec<Score> + Send + Sync;

#[derive(Clone)]
pub struct ProblemParams<C, T>
where
    C: Chromosome,
    T: Clone,
{
    pub codec: Option<Arc<dyn Codec<C, T>>>,
    pub problem: Option<Arc<dyn Problem<C, T>>>,
    pub fitness_fn: Option<Arc<FitnessFn<T>>>,
    pub batch_fitness_fn: Option<Arc<BatchFitnessFn<T>>>,
    pub raw_fitness_fn: Option<Arc<RawFitnessFn<C>>>,
    pub raw_batch_fitness_fn: Option<Arc<RawBatchFitnessFn<C>>>,
}

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    /// Set the codec that will be used to encode and decode the genotype of the population.
    pub fn codec<D: Codec<C, T> + 'static>(mut self, codec: D) -> Self {
        self.params.problem_params.codec = Some(Arc::new(codec));
        self
    }

    /// Set the problem of the genetic engine. This is useful if you want to provide a custom problem.
    pub fn problem<P: Problem<C, T> + 'static>(mut self, problem: P) -> Self {
        self.params.problem_params.problem = Some(Arc::new(problem));
        self
    }

    /// Set the fitness function of the genetic engine. This is the function that will be
    /// used to evaluate the fitness of each individual in the population. This function should
    /// take a single argument of type T and return a [Score]. The [Score] is used to
    /// evaluate or rank the fitness of the individual.
    ///
    /// This method is required and must be set before calling the `build` method.
    pub fn fitness_fn<S: Into<Score>>(
        mut self,
        fitness_func: impl FitnessFunction<T, S> + 'static,
    ) -> Self {
        let other = move |x| fitness_func.evaluate(x).into();
        self.params.problem_params.fitness_fn = Some(Arc::new(other));
        self
    }

    /// Set the batch fitness function of the genetic engine. This function will be used to
    /// evaluate the fitness of a batch of individuals in the population. This function should
    /// take a slice of type `&[T]` and return a `Vec<Score>`. The Score is used to
    /// evaluate or rank the fitness of the individuals.
    ///
    /// This method is optional and can be set after calling the `build` method.
    pub fn batch_fitness_fn<S: Into<Score>>(
        mut self,
        batch_fitness_func: impl BatchFitnessFunction<T, S> + 'static,
    ) -> Self {
        let other = move |x: Vec<T>| {
            batch_fitness_func
                .evaluate(x)
                .into_iter()
                .map(|s| s.into())
                .collect()
        };
        self.params.problem_params.batch_fitness_fn = Some(Arc::new(other));
        self
    }

    pub fn raw_fitness_fn<S: Into<Score>>(
        mut self,
        raw_fitness_func: impl for<'a> FitnessFunction<&'a Genotype<C>, S> + 'static,
    ) -> Self {
        let other = move |x: &Genotype<C>| raw_fitness_func.evaluate(x).into();
        self.params.problem_params.raw_fitness_fn = Some(Arc::new(other));
        self
    }

    pub fn raw_batch_fitness_fn<S: Into<Score>>(
        mut self,
        raw_batch_fitness_func: impl for<'a> BatchFitnessFunction<&'a Genotype<C>, S> + 'static,
    ) -> Self {
        let other = move |x: Vec<&Genotype<C>>| {
            raw_batch_fitness_func
                .evaluate(x)
                .into_iter()
                .map(|s| s.into())
                .collect()
        };
        self.params.problem_params.raw_batch_fitness_fn = Some(Arc::new(other));
        self
    }
}
