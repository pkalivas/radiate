use super::Select;
use crate::objectives::{Objective, Optimize};
use crate::{random_provider, Chromosome, Population};

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

        match objective {
            Objective::Single(opt) => {
                if opt == &Optimize::Minimize {
                    fitness_values.reverse();
                }
            }
            Objective::Multi(_) => {}
        }

        let total_fitness = fitness_values.iter().sum::<f32>();

        for _ in 0..count {
            let mut idx = random_provider::gen_range(0.0..total_fitness);

            for (i, val) in fitness_values.iter().enumerate() {
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
