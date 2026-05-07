use crate::ProbabilityWheelIterator;
use radiate_core::{Chromosome, Objective, Optimize, Population, Select, pareto};

#[derive(Debug, Default)]
pub struct RouletteSelector;

impl RouletteSelector {
    pub fn new() -> Self {
        RouletteSelector
    }
}

impl<C: Chromosome + Clone> Select<C> for RouletteSelector {
    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let weights = match objective {
            Objective::Single(opt) => {
                let mut population_scores = population
                    .iter_scores()
                    .map(|score| score.as_f32())
                    .collect::<Vec<_>>();

                if let Optimize::Minimize = opt {
                    population_scores.reverse();
                }

                population_scores
            }
            Objective::Multi(_) => {
                pareto::weights(&population.iter_scores().collect::<Vec<_>>(), objective)
            }
        };

        ProbabilityWheelIterator::new(&weights, count)
            .map(|idx| population[idx].clone())
            .collect()
    }
}
