use crate::GeneticEngineBuilder;
use radiate_core::{Chromosome, Codec, Problem, fitness::IntoFitness, problem::FitnessKind};
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
    /// take a single argument of type T and return a [Score](radiate_core::Score). The [Score](radiate_core::Score) is used to
    /// evaluate or rank the fitness of the individual.
    pub fn fitness_fn<M>(mut self, fitness_func: impl IntoFitness<C, T, M> + 'static) -> Self {
        let fitness_kind = fitness_func.into_fitness();
        self.params.problem_params.fitness_kind = Some(fitness_kind);
        self
    }
}
