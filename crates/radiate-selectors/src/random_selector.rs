use radiate_core::{Chromosome, Objective, Population, Select, random_provider};

#[derive(Debug, Default)]
pub struct RandomSelector;

impl RandomSelector {
    pub fn new() -> Self {
        RandomSelector
    }
}

impl<C: Chromosome> Select<C> for RandomSelector {
    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        let mut selected = Vec::with_capacity(count);

        for _ in 0..count {
            let member = random_provider::choose(&population.individuals);
            selected.push(member.clone());
        }

        Population::from(selected)
    }
}
