use rand::Rng;

use crate::{Gene, Optimize, Population};

use super::Select;

pub struct RouletteSelector;

impl RouletteSelector {
    pub fn new() -> Self {
        Self
    }
}

impl<G: Gene<G, A>, A> Select<G, A> for RouletteSelector {
    fn select(
        &self,
        population: &Population<G, A>,
        optimize: &Optimize,
        count: usize,
    ) -> Population<G, A> {
        let mut selected = Vec::with_capacity(count);
        let mut fitness_values = Vec::with_capacity(population.len());
        let mut rng = rand::thread_rng();

        let total = population
            .iter()
            .map(|individual| match individual.score() {
                Some(score) => score.as_float(),
                None => 0.0,
            })
            .sum::<f32>();

        for individual in population.iter() {
            let score = match individual.score() {
                Some(score) => score.as_float(),
                None => 0.0,
            };

            fitness_values.push(score / total);
        }

        if optimize == &Optimize::Minimize {
            fitness_values.reverse();
        }

        let total_fitness = fitness_values.iter().sum::<f32>();

        for _ in 0..count {
            let mut idx = rng.gen_range(0.0..total_fitness);

            for i in 0..fitness_values.len() {
                idx -= fitness_values[i];
                if idx <= 0.0 {
                    selected.push(population.get(i).clone());
                    break;
                }
            }
        }

        Population::from_vec(selected)
    }
}