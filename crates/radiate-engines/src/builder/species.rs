use crate::GeneticEngineBuilder;
use radiate_core::{Chromosome, Expr, Phenotype, diversity::Distance};
use std::sync::Arc;

#[derive(Clone)]
pub struct SpeciesParams<C: Chromosome> {
    pub distance: Option<Arc<dyn Distance<Phenotype<C>, Output = f32>>>,
    pub species_threshold: Expr,
    pub max_species_age: usize,
    pub target_species_count: Option<usize>,
}

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    pub fn boxed_diversity(
        mut self,
        diversity: Option<Box<dyn Distance<Phenotype<C>, Output = f32>>>,
    ) -> Self {
        self.params.species_params.distance = diversity.map(|d| d.into());
        self
    }

    pub fn diversity<D: Distance<Phenotype<C>, Output = f32> + 'static>(
        mut self,
        diversity: D,
    ) -> Self {
        self.params.species_params.distance = Some(Arc::new(diversity));
        self
    }

    pub fn species_threshold(mut self, threshold: impl Into<Expr>) -> Self {
        self.params.species_params.species_threshold = threshold.into().compile();
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
