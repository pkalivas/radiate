use radiate_core::{Chromosome, Objective, Optimize, Population, Select, pareto, random_provider};

pub struct StochasticUniversalSamplingSelector;

impl StochasticUniversalSamplingSelector {
    pub fn new() -> Self {
        StochasticUniversalSamplingSelector
    }
}

impl<C: Chromosome> Select<C> for StochasticUniversalSamplingSelector {
    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let fitness_values = match objective {
            Objective::Single(opt) => {
                let scores = population
                    .get_scores()
                    .iter()
                    .map(|score| score.as_f32())
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
                let weights = pareto::weights(&population.get_scores(), objective);
                let total_weights = weights.iter().sum::<f32>();
                weights
                    .iter()
                    .map(|&fit| fit / total_weights)
                    .collect::<Vec<f32>>()
            }
        };

        let fitness_total = fitness_values.iter().sum::<f32>();
        let point_distance = fitness_total / count as f32;
        let start_point = random_provider::range(0.0..point_distance);

        let mut pointers = Vec::with_capacity(count);
        let mut current_point = start_point;

        for _ in 0..count {
            let mut index = 0;
            let mut fitness_sum = fitness_values[index];
            while fitness_sum < current_point && index < fitness_values.len() - 1 {
                index += 1;
                fitness_sum += fitness_values[index];
            }
            pointers.push(population[index].clone());
            current_point += point_distance;
        }

        Population::new(pointers)
    }
}
