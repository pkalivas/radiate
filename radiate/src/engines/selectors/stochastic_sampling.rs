use crate::objectives::Objective;
use crate::{random_provider, Chromosome, Population, Select};

pub struct StochasticUniversalSampling;

impl StochasticUniversalSampling {
    pub fn new() -> Self {
        StochasticUniversalSampling
    }
}

impl<C: Chromosome> Select<C> for StochasticUniversalSampling {
    fn name(&self) -> &'static str {
        "StochasticUniversalSampling"
    }

    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        let total_fitness: f32 = population
            .iter()
            .map(|ind| ind.score().as_ref().unwrap().as_float())
            .sum();
        let point_distance = total_fitness / count as f32;
        let start_point = random_provider::gen_range(0.0..point_distance);

        let mut selected_population = Vec::with_capacity(count);
        let mut cumulative_fitness = 0.0;
        let mut current_point = start_point;

        for individual in population.iter() {
            cumulative_fitness += individual.score().as_ref().unwrap().as_float();
            while cumulative_fitness >= current_point && selected_population.len() < count {
                selected_population.push(individual.clone());
                current_point += point_distance;
            }
        }

        Population::from_vec(selected_population)
    }
}
