use radiate_core::{
    Chromosome, Ecosystem, EngineStep, Front, MetricSet, Phenotype, metric_names, pareto,
};
use std::sync::{Arc, RwLock};

pub struct FrontStep<C: Chromosome> {
    pub(crate) front: Arc<RwLock<Front<Phenotype<C>>>>,
}

impl<C> EngineStep<C> for FrontStep<C>
where
    C: Chromosome + 'static,
{
    fn execute(&mut self, _: usize, metrics: &mut MetricSet, ecosystem: &mut Ecosystem<C>) {
        let timer = std::time::Instant::now();

        let new_individuals = pareto::pareto_front(
            &ecosystem.population().iter().collect::<Vec<_>>(),
            &self.front.read().unwrap().objective(),
        );

        let phenotypes = new_individuals
            .into_iter()
            .map(|pheno| Phenotype::clone(pheno))
            .collect::<Vec<Phenotype<C>>>();
        let count = self.front.write().unwrap().add_all(&phenotypes);

        metrics.upsert_operations(metric_names::FRONT, count as f32, timer.elapsed());
    }
}
