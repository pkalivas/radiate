use radiate_core::{Chromosome, Objective, Phenotype, Select, random_provider};

#[derive(Debug, Default)]
pub struct RandomSelector;

impl RandomSelector {
    pub fn new() -> Self {
        RandomSelector
    }
}

impl<C: Chromosome + Clone> Select<C> for RandomSelector {
    fn select(&self, population: &[Phenotype<C>], _: &Objective, count: usize) -> Vec<usize> {
        let mut selected = Vec::with_capacity(count);

        for _ in 0..count {
            let idx = random_provider::range(0..population.len());
            selected.push(idx);
        }

        selected
    }
}
