use super::Select;
use crate::objectives::{Objective, Optimize};
use crate::selectors::ProbabilityWheelIterator;
use crate::{Chromosome, Population};

pub struct RouletteSelector;

impl RouletteSelector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RouletteSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl<C: Chromosome> Select<C> for RouletteSelector {
    fn name(&self) -> &'static str {
        "Roulette Selector"
    }

    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Population<C> {
        let mut selected = Vec::with_capacity(count);
        let mut fitness_values = Vec::with_capacity(population.len());
        let scores = population
            .iter()
            .map(|individual| individual.score().as_ref().unwrap().as_float())
            .collect::<Vec<f32>>();

        // scale the fitness values so that they sum to 1
        let total = scores.iter().sum::<f32>();
        for fit in scores.iter() {
            fitness_values.push(fit / total);
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

        // Select individuals based on their fitness values
        let prob_iter = ProbabilityWheelIterator::new(&fitness_values, count);
        for idx in prob_iter {
            selected.push(population[idx].clone());
        }

        Population::from_vec(selected)
    }
}
