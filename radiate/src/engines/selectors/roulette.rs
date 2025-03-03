use super::Select;
use crate::objectives::{Objective, Optimize};
use crate::selectors::ProbabilityWheelIterator;
use crate::{Chromosome, EngineError, Population};

pub struct RouletteSelector;

impl RouletteSelector {
    pub fn new() -> Self {
        RouletteSelector
    }
}

impl<C: Chromosome> Select<C> for RouletteSelector {
    fn name(&self) -> &'static str {
        "RouletteSelector"
    }

    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Result<Population<C>, EngineError> {
        let mut selected = Vec::with_capacity(count);
        let mut fitness_values = Vec::with_capacity(population.len());
        let scores = population
            .iter()
            .filter_map(|individual| individual.score())
            .map(|score| score.as_f32())
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
                return Err(EngineError::SelectorError(
                    "Roulette Selector does not support multi-objective optimization.".to_string(),
                ));
            }
        }

        // Select individuals based on their fitness values
        let prob_iter = ProbabilityWheelIterator::new(&fitness_values, count);
        for idx in prob_iter {
            selected.push(population[idx].clone());
        }

        Ok(Population::new(selected))
    }
}
