use crate::GeneticEngineBuilder;
use radiate_core::{Chromosome, Front, Objective, Optimize, Phenotype};
use std::ops::Range;

#[derive(Clone)]
pub struct OptimizeParams<C: Chromosome> {
    pub objectives: Objective,
    pub front_range: Range<usize>,
    pub front: Option<Front<Phenotype<C>>>,
}

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    /// Set the optimization goal of the genetic engine to minimize the fitness function.
    pub fn minimizing(mut self) -> Self {
        self.params.optimization_params.objectives = Objective::Single(Optimize::Minimize);
        self
    }

    /// Set the optimization goal of the genetic engine to maximize the fitness function.
    pub fn maximizing(mut self) -> Self {
        self.params.optimization_params.objectives = Objective::Single(Optimize::Maximize);
        self
    }

    pub fn multi_objective(
        mut self,
        objectives: impl Into<Vec<Optimize>>,
    ) -> GeneticEngineBuilder<C, T> {
        self.params.optimization_params.objectives = Objective::Multi(objectives.into());
        self
    }

    /// Set the minimum and maximum size of the pareto front. This is used for
    /// multi-objective optimization problems where the goal is to find the best
    /// solutions that are not dominated by any other solution.
    pub fn front_size(mut self, range: Range<usize>) -> GeneticEngineBuilder<C, T> {
        self.add_error_if(
            || range.start > range.end,
            "Front range start must be less than end",
        );

        self.params.optimization_params.front_range = range;
        self
    }
}
