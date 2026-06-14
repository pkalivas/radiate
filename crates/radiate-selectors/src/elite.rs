use radiate_core::{Chromosome, Objective, Phenotype, Select};

#[derive(Debug, Default)]
pub struct EliteSelector;

impl EliteSelector {
    pub fn new() -> Self {
        EliteSelector
    }
}

impl<C: Chromosome> Select<C> for EliteSelector {
    fn select(&self, population: &[Phenotype<C>], _: &Objective, count: usize) -> Vec<usize> {
        (0..count.min(population.len())).collect()
    }
}
