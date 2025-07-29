use crate::GeneticEngineBuilder;
use radiate_core::{Chromosome, Codec, Problem, Score, fitness::FitnessFunction};
use std::sync::Arc;

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    /// Set the codec that will be used to encode and decode the genotype of the population.
    pub fn codec<D: Codec<C, T> + 'static>(mut self, codec: D) -> Self {
        self.params.codec = Some(Arc::new(codec));
        self
    }

    /// Set the problem of the genetic engine. This is useful if you want to provide a custom problem.
    pub fn problem<P: Problem<C, T> + 'static>(mut self, problem: P) -> Self {
        self.params.problem = Some(Arc::new(problem));
        self
    }

    /// Set the fitness function of the genetic engine. This is the function that will be
    /// used to evaluate the fitness of each individual in the population. This function should
    /// take a single argument of type T and return a `Score`. The `Score` is used to
    /// evaluate or rank the fitness of the individual.
    ///
    /// This method is required and must be set before calling the `build` method.
    pub fn fitness_fn<S: Into<Score>>(
        mut self,
        fitness_func: impl FitnessFunction<T, S> + 'static,
    ) -> Self {
        let other = move |x| fitness_func.evaluate(x).into();
        self.params.fitness_fn = Some(Arc::new(other));
        self
    }
}
