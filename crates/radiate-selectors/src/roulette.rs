use crate::{ProbabilityWheelIterator, directed_weights};
use radiate_core::{Chromosome, Objective, Phenotype, Select, math::norm, pareto};

#[derive(Debug, Default)]
pub struct RouletteSelector;

impl RouletteSelector {
    pub fn new() -> Self {
        RouletteSelector
    }
}

impl<C: Chromosome> Select<C> for RouletteSelector {
    fn select(
        &self,
        population: &[Phenotype<C>],
        objective: &Objective,
        count: usize,
    ) -> Vec<usize> {
        let fitness_values = match objective {
            Objective::Single(opt) => {
                let mut scores = population
                    .iter()
                    .filter_map(|p| p.score())
                    .filter_map(|s| s.first())
                    .collect::<Vec<_>>();

                directed_weights(&mut scores, *opt);
                norm::scale_l1(&mut scores);

                scores
            }
            Objective::Multi(_) => {
                let scores = population
                    .iter()
                    .filter_map(|p| p.score())
                    .collect::<Vec<_>>();

                let mut weights = pareto::weights(&scores, objective);
                norm::scale_l1(&mut weights);
                weights
            }
        };

        ProbabilityWheelIterator::new(fitness_values, count).collect()
    }
}
