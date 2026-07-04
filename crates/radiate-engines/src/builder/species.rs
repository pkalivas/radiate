use crate::GeneticEngineBuilder;
use radiate_core::{Chromosome, Diversity, Expr};
use std::sync::Arc;

#[derive(Clone)]
pub struct SpeciesParams<C: Chromosome> {
    pub diversity: Option<Arc<dyn Diversity<C>>>,
    pub species_threshold: Expr,
    pub max_species_age: usize,
    pub target_species_count: Option<usize>,
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

    pub fn species_threshold(mut self, threshold: impl Into<Expr>) -> Self {
        self.params.species_params.species_threshold = threshold.into();
        self
    }

    pub fn max_species_age(mut self, max_species_age: usize) -> Self {
        self.add_error_if(
            || max_species_age < 1,
            "max_species_age must be greater than 0",
        );

        self.params.species_params.max_species_age = max_species_age;
        self
    }

    pub fn target_species(mut self, target_species_count: usize) -> Self {
        self.add_error_if(
            || target_species_count < 1,
            "target_species_count must be greater than 0",
        );

        self.params.species_params.target_species_count = Some(target_species_count);
        self
    }
}
