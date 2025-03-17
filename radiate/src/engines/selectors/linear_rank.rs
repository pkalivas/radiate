use crate::objectives::{Objective, Optimize};
use crate::{Chromosome, Population, Select, pareto, random_provider};

pub struct LinearRankSelector {
    selection_pressure: f32,
}

impl LinearRankSelector {
    pub fn new(selection_pressure: f32) -> Self {
        LinearRankSelector { selection_pressure }
    }
}

impl<C: Chromosome> Select<C> for LinearRankSelector {
    fn name(&self) -> &'static str {
        "LinearRankSelector"
    }

    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let fitness_values = match objective {
            Objective::Single(opt) => {
                let scores = population
                    .get_scores_ref()
                    .iter()
                    .map(|scores| scores[0])
                    .collect::<Vec<f32>>();
                let total = scores.iter().sum::<f32>();
                let mut fitness_values =
                    scores.iter().map(|&fit| fit / total).collect::<Vec<f32>>();

                if let Optimize::Minimize = opt {
                    fitness_values.reverse();
                }

                fitness_values
            }
            Objective::Multi(_) => {
                let weights = pareto::weights(&population.get_scores_ref(), objective);
                let total_weights = weights.iter().sum::<f32>();
                weights
                    .iter()
                    .map(|&fit| fit / total_weights)
                    .collect::<Vec<f32>>()
            }
        };

        let total_rank = (1..=fitness_values.len()).map(|i| i as f32).sum::<f32>();
        let mut selected_population = Vec::with_capacity(count);

        for _ in 0..count {
            let target = random_provider::range(0.0..total_rank);
            let mut cumulative_rank = 0.0;

            for (rank, _) in fitness_values.iter().enumerate() {
                cumulative_rank += (rank + 1) as f32 * self.selection_pressure;
                if cumulative_rank > target {
                    selected_population.push(population[rank].clone());
                    break;
                }
            }
        }

        Population::new(selected_population)
    }
}
