use crate::GeneticEngineBuilder;
use radiate_core::{
    Chromosome, Codec, Genotype, Problem, Score,
    fitness::{BatchFitnessFunction, FitnessFunction, IntoFitness},
    problem::FitnessKind,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct ProblemParams<C, T>
where
    C: Chromosome,
    T: Clone,
{
    pub codec: Option<Arc<dyn Codec<C, T>>>,
    pub problem: Option<Arc<dyn Problem<C, T>>>,
    pub fitness_kind: Option<FitnessKind<C, T>>,
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
    // pub fn fitness_fn<S: Into<Score>>(
    //     mut self,
    //     fitness_func: impl FitnessFunction<T, S> + 'static,
    // ) -> Self {
    //     let other = move |x| fitness_func.evaluate(x).into();
    //     self.params.problem_params.fitness_kind = Some(FitnessKind::Single(Arc::new(other)));
    //     self
    // }
    pub fn fitness_fn<M>(mut self, fitness_func: impl IntoFitness<C, T, M> + 'static) -> Self {
        let fitness_kind = fitness_func.into_fitness();
        self.params.problem_params.fitness_kind = Some(fitness_kind);
        self
    }

    /// Set the batch fitness function of the genetic engine. This function will be used to
    /// evaluate the fitness of a batch of individuals in the population. This function should
    /// take a slice of type `&[T]` and return a `Vec<Score>`. The Score is used to
    /// evaluate or rank the fitness of the individuals.
    ///
    /// This method is optional and can be set after calling the `build` method.
    #[deprecated(note = "Use `fitness_fn` instead", since = "1.3.2")]
    pub fn batch_fitness_fn<S: Into<Score>>(
        self,
        batch_fitness_func: impl BatchFitnessFunction<T, S> + 'static,
    ) -> Self {
        self.fitness_fn(batch_fitness_func)
    }

    #[deprecated(note = "Use `fitness_fn` instead", since = "1.3.2")]
    pub fn raw_fitness_fn<S: Into<Score>>(
        mut self,
        raw_fitness_func: impl for<'a> FitnessFunction<&'a Genotype<C>, S> + 'static,
    ) -> Self {
        let other = move |x: &Genotype<C>| raw_fitness_func.evaluate(x).into();
        self.params.problem_params.fitness_kind = Some(FitnessKind::RawSingle(Arc::new(other)));
        self
    }

    #[deprecated(note = "Use `fitness_fn` instead", since = "1.3.2")]
    pub fn raw_batch_fitness_fn<S: Into<Score>>(
        self,
        raw_batch_fitness_func: impl for<'a> BatchFitnessFunction<&'a Genotype<C>, S> + 'static,
    ) -> Self {
        self.fitness_fn(raw_batch_fitness_func)
    }
}
