use super::Select;
use crate::objectives::{Objective, Optimize};
use crate::selectors::ProbabilityWheelIterator;
use crate::{Chromosome, EngineError, Population};

pub struct BoltzmannSelector {
    temperature: f32,
}

impl BoltzmannSelector {
    pub fn new(temperature: f32) -> Self {
        BoltzmannSelector { temperature }
    }
}

impl<C: Chromosome> Select<C> for BoltzmannSelector {
    fn name(&self) -> &'static str {
        "BoltzmannSelector"
    }

    fn select(
        &self,
        population: &Population<C>,
        objective: &Objective,
        count: usize,
    ) -> Result<Population<C>, EngineError> {
        let mut selected = Vec::with_capacity(count);
        let mut min = population[0].score().unwrap().as_f32();
        let mut max = min;

        // Normalize the fitness values.
        for individual in population.iter() {
            let score = individual.score().as_ref().unwrap().as_f32();
            if score < min {
                min = score;
            }
            if score > max {
                max = score;
            }
        }

        let diff = (max - min).abs();
        if diff == 0.0 {
            return Ok(population
                .iter()
                .take(count)
                .cloned()
                .collect::<Population<C>>());
        }

        // Calculate the fitness values for each individual (normalized)
        // and apply the Boltzmann distribution to get the probabilities (temp * fitness).exp()
        let mut result = Vec::with_capacity(population.len());
        for individual in population.iter() {
            let score = individual.score().as_ref().unwrap().as_f32();
            let fitness = (score - min) / diff;
            let value = (self.temperature * fitness).exp();

            result.push(value);
        }

        // Normalize the probabilities to sum to 1
        let total_fitness = result.iter().sum::<f32>();
        for fit in result.iter_mut() {
            *fit /= total_fitness;
        }

        // Reverse the probabilities if minimizing so that the lowest scores have the highest probability
        match objective {
            Objective::Single(opt) => {
                if opt == &Optimize::Minimize {
                    result.reverse();
                }
            }
            Objective::Multi(_) => {
                return Err(EngineError::SelectorError(
                    "BoltzmannSelector only supports single objective optimization".to_string(),
                ));
            }
        }

        // Select the individuals based on the probabilities
        let prob_iter = ProbabilityWheelIterator::new(&result, count);
        for idx in prob_iter {
            selected.push(population[idx].clone());
        }

        Ok(Population::new(selected))
    }
}
