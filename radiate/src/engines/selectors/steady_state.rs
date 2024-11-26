use crate::{random_provider, Chromosome, Objective, Optimize, Population, Select};

pub struct SteadyStateSelector {
    replacement_count: usize,
}

impl SteadyStateSelector {
    pub fn new(replacement_count: usize) -> Self {
        SteadyStateSelector { replacement_count }
    }
}

impl<C: Chromosome> Select<C> for SteadyStateSelector {
    fn name(&self) -> &'static str {
        "SteadyStateSelector"
    }

    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        let mut selected_population = population.clone();
        let slice = population.iter().as_slice();

        for _ in 0..self.replacement_count {
            let new_individual = random_provider::choose(slice).clone();
            let replace_index = random_provider::gen_range(0..selected_population.len());
            selected_population.set(replace_index, new_individual);
        }

        selected_population
            .into_iter()
            .take(count)
            .collect::<Population<C>>()
    }
}
