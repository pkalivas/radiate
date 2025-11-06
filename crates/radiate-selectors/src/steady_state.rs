use radiate_core::{Chromosome, Objective, Population, Select};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SteadyStateSelector {
    replacement_count: usize,
}

impl SteadyStateSelector {
    pub fn new(replacement_count: usize) -> Self {
        SteadyStateSelector { replacement_count }
    }
}

impl<C: Chromosome + Clone> Select<C> for SteadyStateSelector {
    #[allow(unused_variables)]
    fn select(&self, population: &Population<C>, _: &Objective, count: usize) -> Population<C> {
        unimplemented!()
        // let mut selected_population = population.clone();
        // let slice = population.as_ref();

        // for _ in 0..self.replacement_count {
        //     let new_individual = random_provider::choose(slice);
        //     let replace_index = random_provider::range(0..selected_population.len());
        //     selected_population[replace_index] = new_individual.get().clone();
        // }

        // selected_population
        //     .into_iter()
        //     .take(count)
        //     .collect::<Population<C>>()
    }
}
