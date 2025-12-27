use crate::ProbabilityWheelIterator;
use radiate_core::{Chromosome, Objective, Optimize, Population, Select, pareto};

const MIN: f32 = 1e-6;

pub struct BoltzmannSelector {
    temperature: f32,
}

impl BoltzmannSelector {
    pub fn new(temperature: f32) -> Self {
        BoltzmannSelector { temperature }
    }
}

impl<C: Chromosome + Clone> Select<C> for BoltzmannSelector {
    #[inline]
    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let fitness_values = match objective {
            Objective::Single(opt) => {
                let mut scores = Vec::with_capacity(population.len());
                let mut botlzmann_values = Vec::with_capacity(population.len());
                let (mut min, mut max, mut total) = (f32::MAX, f32::MIN, 0.0);

                for score in population.get_scores() {
                    let val = score.as_f32();

                    scores.push(val);
                    min = min.min(val);
                    max = max.max(val);
                }

                let diff = (max - min).abs().max(MIN);
                for &score in scores.iter() {
                    let boltzmann_value = (self.temperature * ((score - min) / diff)).exp();

                    botlzmann_values.push(boltzmann_value);
                    total += boltzmann_value;
                }

                let mut fitness_values = botlzmann_values
                    .iter()
                    .map(|&fit| fit / total)
                    .collect::<Vec<f32>>();

                if let Optimize::Minimize = opt {
                    fitness_values.reverse();
                }

                fitness_values
            }
            Objective::Multi(_) => {
                let weights =
                    pareto::weights(&population.get_scores().collect::<Vec<_>>(), objective);

                let (max, min) = weights.iter().fold((f32::MIN, f32::MAX), |(max, min), &w| {
                    (max.max(w), min.min(w))
                });
                let diff = (max - min).abs().max(MIN);
                let botzmann_values = weights
                    .iter()
                    .map(|&score| (self.temperature * ((score - min) / diff)).exp())
                    .collect::<Vec<f32>>();
                let total_fitness = botzmann_values.iter().sum::<f32>();
                botzmann_values
                    .iter()
                    .map(|&fit| fit / total_fitness)
                    .collect::<Vec<f32>>()
            }
        };

        ProbabilityWheelIterator::new(&fitness_values, count)
            .map(|idx| population[idx].clone())
            .collect::<Population<C>>()
    }
}
