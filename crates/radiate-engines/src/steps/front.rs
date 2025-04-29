use std::sync::{Arc, RwLock};

use radiate_core::{
    Chromosome, Ecosystem, EngineStep, Front, MetricSet, Phenotype, metric_names,
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
    fn execute(
        &mut self,
        generation: usize,
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) {
        let timer = Timer::new();
        let wg = WaitGroup::new();

        let new_individuals = ecosystem
            .population
            .iter()
            .filter(|pheno| pheno.generation() == generation)
            .map(|pheno| pheno.clone())
            .collect::<Vec<Phenotype<C>>>();

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
                    remove_vector
                        .write()
                        .unwrap()
                        .extend(to_remove.iter().map(Arc::clone));
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
            .map(|(idx, _)| &new_individuals[idx])
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

//     /// Updates the front of the population using the scores of the individuals. The front is a collection
//     /// of individuals that are not dominated by any other individual in the population. This method is only
//     /// called if the objective is multi-objective, as the front is not relevant for single-objective optimization.
//     /// The front is updated in a separate thread to avoid blocking the main thread while the front is being calculated.
//     /// This can significantly speed up the calculation of the front for large populations.
//     fn update_front(&self, ctx: &mut EngineContext<C, T>) {
//         let objective = self.params.objective();
//         let thread_pool = self.params.thread_pool();

//         if let Objective::Multi(_) = objective {
//             // TODO: Examine the clones here - it seems like we can reduce the number of clones of
//             // the population. The front is a cheap clone (the values are wrapped in an Arc), but
//             // the population is not. But at the same time - this is still pretty damn fast.
//             let timer = Timer::new();
//             let wg = WaitGroup::new();

//             let new_individuals = ctx
//                 .population
//                 .iter()
//                 .filter(|pheno| pheno.generation() == ctx.index)
//                 .collect::<Vec<&Phenotype<C>>>();

//             let front = Arc::new(RwLock::new(ctx.front.clone()));
//             let dominates_vector = Arc::new(RwLock::new(vec![false; new_individuals.len()]));
//             let remove_vector = Arc::new(RwLock::new(Vec::new()));

//             for (idx, member) in new_individuals.iter().enumerate() {
//                 let pheno = Phenotype::clone(member);
//                 let front_clone = Arc::clone(&front);
//                 let doms_vector = Arc::clone(&dominates_vector);
//                 let remove_vector = Arc::clone(&remove_vector);

//                 thread_pool.group_submit(&wg, move || {
//                     let (dominates, to_remove) = front_clone.read().unwrap().dominates(&pheno);

//                     if dominates {
//                         doms_vector.write().unwrap().get_mut(idx).map(|v| *v = true);
//                         remove_vector
//                             .write()
//                             .unwrap()
//                             .extend(to_remove.iter().map(Arc::clone));
//                     }
//                 });
//             }

//             let count = wg.wait();

//             let dominates_vector = dominates_vector
//                 .read()
//                 .unwrap()
//                 .iter()
//                 .enumerate()
//                 .filter(|(_, is_dominating)| **is_dominating)
//                 .map(|(idx, _)| new_individuals[idx])
//                 .collect::<Vec<&Phenotype<C>>>();
//             let mut remove_vector = remove_vector.write().unwrap();

//             remove_vector.dedup();

//             ctx.front.clean(dominates_vector, remove_vector.as_slice());

//             ctx.upsert_operation(metric_names::FRONT, count as f32, timer);
//         }
//     }
