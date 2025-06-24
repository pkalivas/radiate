use radiate_core::{Chromosome, Ecosystem, EngineStep, Front, MetricSet, Phenotype, metric_names};
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
    ) {
        let timer = std::time::Instant::now();

        // TODO: Check this out - these two varients are comparable.

        // let new_individuals = pareto::pareto_front(
        //     &ecosystem.population().iter().collect::<Vec<_>>(),
        //     &self.front.read().unwrap().objective(),
        // );

        // let phenotypes = new_individuals
        //     .into_iter()
        //     .map(|pheno| Phenotype::clone(pheno))
        //     .collect::<Vec<Phenotype<C>>>();

        let phenotypes = ecosystem
            .population()
            .iter()
            .filter(|ind| ind.age(generation) == 0)
            .cloned()
            .collect::<Vec<Phenotype<C>>>();

        let count = self.front.write().unwrap().add_all(&phenotypes);

        metrics.upsert_operations(metric_names::FRONT, count as f32, timer.elapsed());
    }
}
