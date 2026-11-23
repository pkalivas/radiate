use crate::EvolutionContext;
use crate::steps::{EcosystemView, EngineStep};
use radiate_core::{Chromosome, Front, Phenotype, metric_names};
use radiate_error::Result;
use std::sync::{Arc, RwLock};

const ENTROPY_UPDATE_INTERVAL: usize = 10;

pub struct FrontStep<C: Chromosome> {
    pub(crate) front: Arc<RwLock<Front<Phenotype<C>>>>,
}

impl<C, T> EngineStep<C, T> for FrontStep<C>
where
    C: Chromosome + PartialEq + Clone + 'static,
{
    type View<'a>
        = EcosystemView<'a, C>
    where
        C: 'a,
        T: 'a;

    fn view<'a>(ctx: &'a mut EvolutionContext<C, T>) -> Self::View<'a> {
        EcosystemView::new(ctx)
    }

    #[inline]
    fn execute(&mut self, view: EcosystemView<'_, C>) -> Result<()> {
        let EcosystemView {
            index: generation,
            ecosystem,
            metrics,
        } = view;

        let phenotypes = ecosystem
            .population()
            .iter()
            .filter(|ind| ind.age(generation) == 0);

        let add_result = self.front.write().unwrap().try_add_all(phenotypes);

        metrics.upsert(metric_names::FRONT_ADDITIONS, add_result.added_count);
        metrics.upsert(metric_names::FRONT_REMOVALS, add_result.removed_count);
        metrics.upsert(metric_names::FRONT_COMPARISONS, add_result.comparisons);
        metrics.upsert(metric_names::FRONT_FILTERS, add_result.filter_count);
        metrics.upsert(metric_names::FRONT_SIZE, add_result.size);

        if add_result.added_count > 0 {
            // Update entropy metric every 10 generations - this is an expensive operation so we
            // don't want to do it every generation.
            if generation.is_multiple_of(ENTROPY_UPDATE_INTERVAL) {
                let mut reader = self.front.write().unwrap();
                if let Some(entropy) = reader.entropy() {
                    metrics.upsert(metric_names::FRONT_ENTROPY, entropy);
                }
            }
        }

        Ok(())
    }
}
