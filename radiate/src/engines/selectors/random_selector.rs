use super::Select;
use crate::objectives::Objective;
use crate::{random_provider, Chromosome, EngineCompoment, Population};

pub struct RandomSelector;

impl RandomSelector {
    pub fn new() -> Self {
        Self
    }
}

impl EngineCompoment for RandomSelector {
    fn name(&self) -> &'static str {
        "RandomSelector"
    }
}

impl<C: Chromosome> Select<C> for RandomSelector {
    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        let mut selected = Vec::with_capacity(count);

        for _ in 0..count {
            let index = random_provider::gen_range(0..population.len());
            selected.push(population[index].clone());
        }

        Population::from_vec(selected)
    }
}

impl Default for RandomSelector {
    fn default() -> Self {
        Self::new()
    }
}
