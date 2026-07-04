use crate::GeneticEngineBuilder;
use radiate_core::{Chromosome, EcosystemFilter};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct FilterParams<C: Chromosome> {
    pub filters: Vec<Arc<Mutex<dyn EcosystemFilter<C>>>>,
}

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    /// Add a filter to the engine. The filter will be applied to the [Ecosystem] after the
    /// [RecombineStep] and before the [FrontStep]. This allows you to filter
    /// the population based on custom criteria, such as age, fitness, or any other metric.
    pub fn filter<F: EcosystemFilter<C> + 'static>(mut self, filter: F) -> Self {
        self.params
            .filter_params
            .filters
            .push(Arc::new(Mutex::new(filter)));
        self
    }

    pub fn filters(mut self, filters: Vec<Arc<Mutex<dyn EcosystemFilter<C>>>>) -> Self {
        self.params.filter_params.filters.extend(filters);
        self
    }
}
