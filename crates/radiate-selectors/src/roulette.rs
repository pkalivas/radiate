use crate::ProbabilityWheelIterator;
use radiate_core::{Chromosome, Objective, Optimize, Population, Select, pareto};

const MIN: f32 = 1e-6;

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
                let mut weights = Vec::with_capacity(population.len());

                let mut min = f32::MAX;
                let mut max = f32::MIN;

                for score in population.iter_scores() {
                    let single_score = score.as_f32();

                    weights.push(single_score);
                    min = min.min(single_score);
                    max = max.max(single_score);
                }

                for fit in weights.iter_mut() {
                    *fit = (*fit - min).max(MIN) / (max - min).max(MIN);
                }

                if let Optimize::Minimize = opt {
                    weights.reverse();
                }

                weights
            }
            Objective::Multi(_) => {
                let mut weights =
                    pareto::weights(&population.iter_scores().collect::<Vec<_>>(), objective);
                let total_weights = weights.iter().sum::<f32>();

                for fit in weights.iter_mut() {
                    *fit /= total_weights;
                }

                weights
            }
        };

        ProbabilityWheelIterator::new(&weights, count)
            .map(|idx| population[idx].clone())
            .collect()
    }
}
