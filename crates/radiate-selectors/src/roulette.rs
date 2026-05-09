use crate::ProbabilityWheelIterator;
use radiate_core::{Chromosome, Objective, Optimize, Population, Select, math::norm, pareto};

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
    ) -> Vec<usize> {
        let fitness_values = match objective {
            Objective::Single(opt) => {
                let mut scores = population
                    .iter_scores()
                    .filter_map(|s| s.first())
                    .collect::<Vec<_>>();

                norm::scale_l1_affine_sorted(&mut scores);

                if let Optimize::Minimize = opt {
                    scores.reverse();
                }

                scores
            }
            Objective::Multi(_) => {
                let scores = population.iter_scores().collect::<Vec<_>>();

                let mut weights = pareto::weights(&scores, objective);
                norm::scale_l1(&mut weights);
                weights
            }
        };

        ProbabilityWheelIterator::new(&fitness_values, count).collect()
    }
}
