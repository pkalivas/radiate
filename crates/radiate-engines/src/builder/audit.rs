use crate::GeneticEngineBuilder;
use radiate_core::{Audit, Chromosome, Epoch};
use std::sync::Arc;

impl<C, T, E> GeneticEngineBuilder<C, T, E>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
    E: Epoch<C>,
{
    /// Add a single audit to the algorithm that will produce additional metrics
    /// to collect during the evolution process.
    pub fn audit(mut self, audit: impl Audit<C> + 'static) -> Self {
        self.params.audits.push(Arc::new(audit));
        self
    }

    /// Add a list of audits to the algorithm that will produce additional metrics
    /// to collect during the evolution process.
    pub fn audits(mut self, audits: Vec<Arc<dyn Audit<C>>>) -> Self {
        self.params.audits.extend(audits);
        self
    }
}
