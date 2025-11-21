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
        let fitness_values = match objective {
            Objective::Single(opt) => {
                let mut population_scores = Vec::with_capacity(population.len());
                let mut sum = 0.0;
                for score in population.get_scores() {
                    let single_score = score.as_f32();
                    population_scores.push(single_score);
                    sum += single_score;
                }

                for fit in population_scores.iter_mut() {
                    *fit /= sum;
                }

                if let Optimize::Minimize = opt {
                    population_scores.reverse();
                }

                population_scores
            }
            Objective::Multi(_) => {
                let mut weights =
                    pareto::weights(&population.get_scores().collect::<Vec<_>>(), objective);
                let total_weights = weights.iter().sum::<f32>();

                for fit in weights.iter_mut() {
                    *fit /= total_weights;
                }

                weights
            }
        };

        ProbabilityWheelIterator::new(&fitness_values, count)
            .map(|idx| population[idx].clone())
            .collect()
    }
}
