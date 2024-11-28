use super::Select;
use crate::objectives::{Objective, Optimize};
use crate::selectors::ProbabilityIterator;
use crate::{Chromosome, Population};

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
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let mut selected = Vec::with_capacity(count);
        let mut min = population[0].score().as_ref().unwrap().as_float();
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

        let diff = (max - min).abs();
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
        for fit in result.iter_mut() {
            *fit /= total_fitness;
        }

        if let Objective::Single(opt) = objective {
            if opt == &Optimize::Minimize {
                result.reverse();
            }
        }

        let prob_iter = ProbabilityIterator::new(&result, count);
        for idx in prob_iter {
            selected.push(population[idx].clone());
        }

        Population::from_vec(selected)
    }
}
