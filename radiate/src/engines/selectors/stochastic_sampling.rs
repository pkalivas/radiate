use crate::objectives::{Objective, Optimize};
use crate::{random_provider, Chromosome, Population, Select};

pub struct StochasticUniversalSamplingSelector;

impl StochasticUniversalSamplingSelector {
    pub fn new() -> Self {
        StochasticUniversalSamplingSelector
    }
}

impl<C: Chromosome> Select<C> for StochasticUniversalSamplingSelector {
    fn name(&self) -> &'static str {
        "StochasticUniversalSampling"
    }

    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let mut fitness_values = Vec::with_capacity(population.len());

        let total_fitness: f32 = population
            .iter()
            .map(|ind| ind.score().as_ref().unwrap().as_float())
            .sum();

        for individual in population.iter() {
            let score = individual.score().as_ref().unwrap().as_float();
            fitness_values.push(score / total_fitness);
        }

        match objective {
            Objective::Single(opt) => {
                if opt == &Optimize::Minimize {
                    fitness_values.reverse();
                }
            }
            Objective::Multi(_) => {}
        }

        let fitness_total = fitness_values.iter().sum::<f32>();
        let point_distance = fitness_total / count as f32;
        let start_point = random_provider::gen_range(0.0..point_distance);

        let mut pointers = Vec::with_capacity(count);
        let mut current_point = start_point;

        for _ in 0..count {
            let mut index = 0;
            let mut fitness_sum = fitness_values[index];
            while fitness_sum < current_point {
                index += 1;
                fitness_sum += fitness_values[index];
            }
            pointers.push(population[index].clone());
            current_point += point_distance;
        }

        Population::from_vec(pointers)
    }
}

impl Default for StochasticUniversalSamplingSelector {
    fn default() -> Self {
        Self::new()
    }
}
