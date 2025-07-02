use crate::GeneticEngineBuilder;
use radiate_core::{Chromosome, Diversity};
use radiate_error::radiate_err;
use std::sync::Arc;

#[derive(Clone)]
pub struct SpeciesParams<C: Chromosome> {
    pub diversity: Option<Arc<dyn Diversity<C>>>,
    pub species_threshold: f32,
    pub max_species_age: usize,
}

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    pub fn boxed_diversity(mut self, diversity: Option<Box<dyn Diversity<C>>>) -> Self {
        self.params.species_params.diversity = diversity.map(|d| d.into());
        self
    }

    pub fn diversity<D: Diversity<C> + 'static>(mut self, diversity: D) -> Self {
        self.params.species_params.diversity = Some(Arc::new(diversity));
        self
    }

    pub fn species_threshold(mut self, threshold: f32) -> Self {
        if threshold < 0.0 {
            self.errors
                .push(radiate_err!(InvalidConfig: "species_threshold must be greater than 0"));
        }

        self.params.species_params.species_threshold = threshold;
        self
    }

    pub fn max_species_age(mut self, max_species_age: usize) -> Self {
        if max_species_age < 1 {
            self.errors.push(radiate_err!(
                InvalidConfig: "max_species_age must be greater than 0"
            ));
        }

        self.params.species_params.max_species_age = max_species_age;
        self
    }
}
