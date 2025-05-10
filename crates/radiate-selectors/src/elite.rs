use radiate_core::{Chromosome, Objective, Population, Select};

#[derive(Debug, Default)]
pub struct EliteSelector;

impl EliteSelector {
    pub fn new() -> Self {
        EliteSelector
    }
}

impl<C: Chromosome> Select<C> for EliteSelector {
    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        population.iter().take(count).cloned().collect()
    }
}
