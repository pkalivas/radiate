use std::sync::{Arc, RwLock};

use radiate_core::{
    Chromosome, Ecosystem, EngineStep, Front, MetricSet, Phenotype, metric_names, pareto,
    thread_pool::{ThreadPool, WaitGroup},
    timer::Timer,
};

pub struct FrontStep<C: Chromosome> {
    pub(crate) front: Arc<RwLock<Front<Phenotype<C>>>>,
    pub(crate) thread_pool: Arc<ThreadPool>,
}

impl<C> EngineStep<C> for FrontStep<C>
where
    C: Chromosome + 'static,
{
    fn execute(&mut self, _: usize, metrics: &mut MetricSet, ecosystem: &mut Ecosystem<C>) {
        // let mut writer = self.front.write().unwrap();
        // for member in ecosystem.population().iter() {
        //     writer.add(&member);
        // }

        // if writer.values().len() > writer.range().end {
        //     writer.filter();
        // }

        // drop(writer);
        // return;
        let timer = Timer::new();
        let wg = WaitGroup::new();

        // let new_individuals = ecosystem
        //     .population
        //     .iter()
        //     .filter(|pheno| pheno.generation() == generation)
        //     .map(|pheno| pheno.clone())
        //     .collect::<Vec<Phenotype<C>>>();

        let new_individuals = pareto::pareto_front(
            &ecosystem.population().iter().collect::<Vec<_>>(),
            &self.front.read().unwrap().objective(),
        );

        let front = Arc::clone(&self.front);
        let dominates_vector = Arc::new(RwLock::new(vec![false; new_individuals.len()]));
        let remove_vector = Arc::new(RwLock::new(Vec::new()));

        for (idx, member) in new_individuals.iter().enumerate() {
            let pheno = Phenotype::clone(member);
            let front_clone = Arc::clone(&front);
            let doms_vector = Arc::clone(&dominates_vector);
            let remove_vector = Arc::clone(&remove_vector);

            self.thread_pool.group_submit(&wg, move || {
                let (dominates, to_remove) = front_clone.read().unwrap().dominates(&pheno);

                if dominates {
                    doms_vector.write().unwrap().get_mut(idx).map(|v| *v = true);
                    remove_vector.write().unwrap().extend(to_remove.into_iter());
                }
            });
        }

        let count = wg.wait();

        let dominates_vector = dominates_vector
            .read()
            .unwrap()
            .iter()
            .enumerate()
            .filter(|(_, is_dominating)| **is_dominating)
            .map(|(idx, _)| new_individuals[idx])
            .collect::<Vec<&Phenotype<C>>>();
        let mut remove_vector = remove_vector.write().unwrap();

        remove_vector.dedup();

        self.front
            .write()
            .unwrap()
            .clean(dominates_vector, remove_vector.as_slice());

        metrics.upsert_operations(metric_names::FRONT, count as f32, timer);
    }
}
