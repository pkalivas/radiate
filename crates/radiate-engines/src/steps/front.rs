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
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) -> Result<()> {
        let timer = std::time::Instant::now();

        let phenotypes = ecosystem
            .population()
            .iter()
            .filter(|ind| ind.age(generation) == 0)
            .cloned()
            .collect::<Vec<Phenotype<C>>>();

        let count = self.front.write().unwrap().add_all(&phenotypes);

        if count > 0 {
            metrics.upsert(metric_names::FRONT_ADDITIONS, (count, timer.elapsed()));
        }

        Ok(())
    }
}
