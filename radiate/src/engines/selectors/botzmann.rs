use crate::{Gene, Optimize, Population, RandomProvider};

use super::Select;

pub struct BoltzmannSelector {
    temperature: f32,
}

impl BoltzmannSelector {
    pub fn new(temperature: f32) -> Self {
        Self { temperature }
    }
}

impl<G: Gene<G, A>, A> Select<G, A> for BoltzmannSelector {
    fn name(&self) -> &'static str {
        "Boltzmann Selector"
    }

    fn select(
        &self,
        population: &Population<G, A>,
        optimize: &Optimize,
        count: usize,
    ) -> Population<G, A> {
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
                .map(|individual| individual.clone())
                .collect::<Population<G, A>>();
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
            let mut idx = RandomProvider::gen_range(0.0..total_fitness);

            for i in 0..result.len() {
                idx -= result[i];
                if idx <= 0.0 {
                    selected.push(population.get(i).clone());
                    break;
                }
            }
        }

        Population::from_vec(selected)
    }
}
