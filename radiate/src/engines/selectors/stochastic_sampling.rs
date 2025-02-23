use crate::objectives::{Objective, Optimize};
use crate::{Chromosome, EngineCompoment, Population, Select, random_provider};

pub struct StochasticUniversalSamplingSelector;

impl StochasticUniversalSamplingSelector {
    pub fn new() -> Self {
        StochasticUniversalSamplingSelector
    }
}

impl EngineCompoment for StochasticUniversalSamplingSelector {
    fn name(&self) -> &'static str {
        "StochasticUniversalSamplingSelector"
    }
}

impl<C: Chromosome> Select<C> for StochasticUniversalSamplingSelector {
    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let mut fitness_values = Vec::with_capacity(population.len());

        let total_fitness = population
            .iter()
            .filter_map(|ind| ind.score())
            .map(|score| score.as_f32())
            .sum::<f32>();

        for individual in population.iter() {
            let score = individual.score().as_ref().unwrap().as_f32();
            fitness_values.push(score / total_fitness);
        }

        match objective {
            Objective::Single(opt) => {
                if opt == &Optimize::Minimize {
                    fitness_values.reverse();
                }
            }
            Objective::Multi(_) => {
                panic!("Multi-objective optimization is not supported by this selector.");
            }
        }

        let fitness_total = fitness_values.iter().sum::<f32>();
        let point_distance = fitness_total / count as f32;
        let start_point = random_provider::random_range(0.0..point_distance);

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
