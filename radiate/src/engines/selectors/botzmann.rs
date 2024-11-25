use crate::{random_provider, Chromosome, Optimize, Population};

use super::Select;

pub struct BoltzmannSelector {
    temperature: f32,
}

impl BoltzmannSelector {
    pub fn new(temperature: f32) -> Self {
        Self { temperature }
    }
}

impl<C: Chromosome> Select<C> for BoltzmannSelector {
    fn name(&self) -> &'static str {
        "Boltzmann Selector"
    }

    fn select(
        &self,
        population: &Population<C>,
        optimize: &Optimize,
        count: usize,
    ) -> Population<C> {
        let mut selected = Vec::with_capacity(count);

        let mut min = population.get(0).score().as_ref().unwrap().as_float();
        let mut max = min;

        for individual in population.iter() {
            let score = individual.score().as_ref().unwrap().as_float();
            if score < min {
                min = score;
            }
            if score > max {
                max = score;
            }
        }

        let diff = max - min;
        if diff == 0.0 {
            return population
                .iter()
                .take(count)
                .cloned()
                .collect::<Population<C>>();
        }

        let mut result = Vec::with_capacity(population.len());
        for individual in population.iter() {
            let score = individual.score().as_ref().unwrap().as_float();
            let fitness = (score - min) / diff;
            let value = (self.temperature * fitness).exp();

            result.push(value);
        }

        let total_fitness = result.iter().sum::<f32>();
        for i in 0..result.len() {
            result[i] /= total_fitness;
        }

        if optimize == &Optimize::Minimize {
            result.reverse();
        }

        let total_fitness = result.iter().sum::<f32>();

        for _ in 0..count {
            let mut idx = random_provider::gen_range(0.0..total_fitness);

            for (i, val) in result.iter().enumerate() {
                idx -= val;
                if idx <= 0.0 {
                    selected.push(population.get(i).clone());
                    break;
                }
            }
        }

        Population::from_vec(selected)
    }
}
