use super::Select;
use crate::objectives::Objective;
use crate::{random_provider, Chromosome, Population};

pub struct MonteCarloSelector;

impl MonteCarloSelector {
    pub fn new() -> Self {
        Self
    }
}

impl<C: Chromosome> Select<C> for MonteCarloSelector {
    fn name(&self) -> &'static str {
        "Monte Carlo Selector"
    }

    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        let mut selected = Vec::with_capacity(count);

        for _ in 0..count {
            let index = random_provider::gen_range(0..population.len());
            selected.push(population[index].clone());
        }

        Population::from_vec(selected)
    }
}
