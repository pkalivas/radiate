use radiate_core::{
    Chromosome, Objective, Optimize, Population, ProbabilityWheelIterator, Select, pareto,
};

pub struct BoltzmannSelector {
    temperature: f32,
}

impl BoltzmannSelector {
    pub fn new(temperature: f32) -> Self {
        BoltzmannSelector { temperature }
    }
}

impl<C: Chromosome> Select<C> for BoltzmannSelector {
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

                let (min, max) = scores
                    .iter()
                    .fold((f32::MAX, f32::MIN), |(min, max), &score| {
                        (min.min(score), max.max(score))
                    });
                let diff = (max - min).abs().max(1e-6);
                let botzlmann_values = scores
                    .iter()
                    .map(|&score| (self.temperature * ((score - min) / diff)).exp())
                    .collect::<Vec<f32>>();

                let total_fitness = botzlmann_values.iter().sum::<f32>();
                let mut fitness_values = botzlmann_values
                    .iter()
                    .map(|&fit| fit / total_fitness)
                    .collect::<Vec<f32>>();

                if let Optimize::Minimize = opt {
                    fitness_values.reverse();
                }

                fitness_values
            }
            Objective::Multi(_) => {
                let weights = pareto::weights(&population.get_scores(), objective);

                let (max, min) = weights.iter().fold((f32::MIN, f32::MAX), |(max, min), &w| {
                    (max.max(w), min.min(w))
                });
                let diff = (max - min).abs().max(1e-6);
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
