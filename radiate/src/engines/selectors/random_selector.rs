use super::Select;
use crate::objectives::Objective;
use crate::{Chromosome, Population, EngineError, random_provider};

pub struct RandomSelector;

impl RandomSelector {
    pub fn new() -> Self {
        RandomSelector
    }
}

impl<C: Chromosome> Select<C> for RandomSelector {
    fn name(&self) -> &'static str {
        "RandomSelector"
    }

    fn select(
        &self,
        population: &Population<C>,
        _: &Objective,
        count: usize,
    ) -> Result<Population<C>, EngineError> {
        let mut selected = Vec::with_capacity(count);

        for _ in 0..count {
            let index = random_provider::random_range(0..population.len());
            selected.push(population[index].clone());
        }

        Ok(Population::new(selected))
    }
}

impl Default for RandomSelector {
    fn default() -> Self {
        Self::new()
    }
}
