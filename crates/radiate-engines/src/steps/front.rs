use crate::steps::EngineStep;
use radiate_core::{Chromosome, Ecosystem, Front, MetricSet, Phenotype, metric_names};
use radiate_error::Result;
use std::sync::{Arc, RwLock};

pub struct FrontStep<C: Chromosome> {
    pub(crate) front: Arc<RwLock<Front<Phenotype<C>>>>,
}

impl<C> EngineStep<C> for FrontStep<C>
where
    C: Chromosome + PartialEq + Clone + 'static,
{
    #[inline]
    fn execute(
        &mut self,
        generation: usize,
        ecosystem: &mut Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Result<()> {
        let phenotypes = ecosystem
            .population()
            .iter()
            .filter(|ind| ind.age(generation) == 0)
            .cloned()
            .collect::<Vec<Phenotype<C>>>();

        let add_result = self.front.write().unwrap().add_all(phenotypes);

        metrics.upsert((metric_names::FRONT_ADDITIONS, add_result.added_count));
        metrics.upsert((metric_names::FRONT_REMOVALS, add_result.removed_count));
        metrics.upsert((metric_names::FRONT_COMPARISONS, add_result.comparisons));
        metrics.upsert((metric_names::FRONT_FILTERS, add_result.filter_count));
        metrics.upsert((metric_names::FRONT_SIZE, add_result.size));

        if add_result.added_count > 0 {
            if generation % 10 == 0 {
                let reader = self.front.read().unwrap();
                if let Some(entropy) = reader.entropy() {
                    metrics.upsert((metric_names::FRONT_ENTROPY, entropy));
                }
            }
        }

        Ok(())
    }
}
