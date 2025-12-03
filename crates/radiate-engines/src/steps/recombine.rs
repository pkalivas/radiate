use crate::steps::EngineStep;
use radiate_core::{
    Alter, Chromosome, Ecosystem, MetricSet, Objective, Optimize, Population, Select,
};
use radiate_error::Result;
use std::sync::Arc;

pub struct RecombineStep<C: Chromosome> {
    pub(crate) survivor_handle: SurvivorRecombineHandle<C>,
    pub(crate) offspring_handle: OffspringRecombineHandle<C>,
}

#[derive(Clone)]
pub struct SurvivorRecombineHandle<C: Chromosome> {
    pub(crate) count: usize,
    pub(crate) objective: Objective,
    pub(crate) selector: Arc<dyn Select<C>>,
}

impl<C> SurvivorRecombineHandle<C>
where
    C: Chromosome + Clone,
{
    #[inline]
    pub fn select(&self, population: &Population<C>, metrics: &mut MetricSet) -> Population<C> {
        let time = std::time::Instant::now();
        let survivors = self
            .selector
            .select(&population, &self.objective, self.count);

        metrics.upsert(self.selector.name(), (survivors.len(), time.elapsed()));
        survivors
    }
}

#[derive(Clone)]
pub struct OffspringRecombineHandle<C: Chromosome> {
    pub(crate) count: usize,
    pub(crate) objective: Objective,
    pub(crate) selector: Arc<dyn Select<C>>,
    pub(crate) alters: Vec<Arc<dyn Alter<C>>>,
}

impl<C> OffspringRecombineHandle<C>
where
    C: Chromosome + PartialEq + Clone,
{
    #[inline]
    pub fn create(
        &self,
        generation: usize,
        ecosystem: &Ecosystem<C>,
        metrics: &mut MetricSet,
    ) -> Population<C> {
        if let Some(species) = ecosystem.species() {
            let total_offspring = self.count as f32;
            let mut species_scores = species
                .iter()
                .filter_map(|spec| spec.score())
                .collect::<Vec<_>>();

            if let Objective::Single(Optimize::Minimize) = &self.objective {
                species_scores.reverse();
            }

            let mut next_population = Vec::with_capacity(self.count);

            for (species, score) in species.iter().zip(species_scores.iter()) {
                let count = (score.as_f32() * total_offspring).round() as usize;
                let time = std::time::Instant::now();
                let mut offspring =
                    self.selector
                        .select(&species.population(), &self.objective, count);
                metrics.upsert(self.selector.name(), (offspring.len(), time.elapsed()));

                self.objective.sort(&mut offspring);

                self.alters.iter().for_each(|alt| {
                    alt.alter(&mut offspring, generation)
                        .into_iter()
                        .for_each(|metric| {
                            metrics.add_or_update(metric);
                        });
                });

                next_population.extend(offspring);
            }

            Population::new(next_population)
        } else {
            let timer = std::time::Instant::now();
            let mut offspring =
                self.selector
                    .select(&ecosystem.population(), &self.objective, self.count);

            metrics.upsert(self.selector.name(), (offspring.len(), timer.elapsed()));

            self.objective.sort(&mut offspring);

            self.alters.iter().for_each(|alt| {
                alt.alter(&mut offspring, generation)
                    .into_iter()
                    .for_each(|metric| {
                        metrics.add_or_update(metric);
                    });
            });

            offspring
        }
    }
}

impl<C> EngineStep<C> for RecombineStep<C>
where
    C: Chromosome + PartialEq + Clone,
{
    #[inline]
    fn execute(
        &mut self,
        generation: usize,
        metrics: &mut MetricSet,
        ecosystem: &mut Ecosystem<C>,
    ) -> Result<()> {
        let survivors = self
            .survivor_handle
            .select(&ecosystem.population(), metrics);
        let offspring = self.offspring_handle.create(generation, ecosystem, metrics);

        ecosystem.population_mut().clear();
        ecosystem.population_mut().extend(survivors);
        ecosystem.population_mut().extend(offspring);

        Ok(())
    }
}

// // Immutable views we can safely share across threads
// let eco_ref = &*ecosystem;
// let surv_handle_ref = self.survivor_handle.clone();
// let off_handle_ref = self.offspring_handle.clone();

// // Per-branch metrics to avoid sharing &mut MetricSet across threads
// let mut survivor_metrics = MetricSet::new();
// let mut offspring_metrics = MetricSet::new();

// // let (survivors, offspring) = thread::scope(|scope| {
// //     // Spawn survivor selection thread
// //     let surv_join = scope
// //         .spawn(|_| surv_handle_ref.select(&eco_ref.population(), &mut survivor_metrics));

// //     // Spawn offspring creation thread
// //     let off_join =
// //         scope.spawn(|_| off_handle_ref.create(generation, eco_ref, &mut offspring_metrics));

// //     let survivors = surv_join.join().expect("survivor thread panicked");
// //     let offspring = off_join.join().expect("offspring thread panicked");

// //     (survivors, offspring)
// // });
// // let pool = radiate_core::domain::get_thread_pool(10);

// let surv_eco = Ecosystem::clone_ref(ecosystem);
// let off_eco = Ecosystem::clone_ref(ecosystem);
// let one_seed = random_provider::random::<u64>();
// let two_seed = random_provider::random::<u64>();
// let surv_task: Box<dyn FnOnce() -> Population<C> + Send> = Box::new({
//     // random_provider::scoped_seed(one_seed, || {
//     move || {
//         let time = std::time::Instant::now();
//         random_provider::seed_current_thread(one_seed);
//         let surv = surv_handle_ref.select(&surv_eco.population(), &mut MetricSet::new());
//         println!("Survivor selection took: {:?}", time.elapsed());
//         surv
//     }
//     // })
// });

// let off_task: Box<dyn FnOnce() -> Population<C> + Send> = Box::new({
//     move || {
//         let time = std::time::Instant::now();
//         random_provider::seed_current_thread(two_seed);
//         let off = off_handle_ref.create(generation, &off_eco, &mut MetricSet::new());
//         println!("Offspring creation took: {:?}", time.elapsed());
//         off
//     }
// });

// let mut t = self.executor.execute_batch(vec![off_task, surv_task]);
// // let surv_work = radiate_core::domain::get_thread_pool(10).submit_with_result(surv_task);
// // let off_work = radiate_core::domain::get_thread_pool(10).submit_with_result(off_task);

// let survivors = t.pop().unwrap();
// let offspring = t.pop().unwrap();

// // Merge branch metrics into the main MetricSet
// survivor_metrics.flush_all_into(metrics);
// offspring_metrics.flush_all_into(metrics);

// // Replace population with survivors + offspring
// let pop = ecosystem.population_mut();
// pop.clear();
// pop.extend(survivors);
// pop.extend(offspring);

// Ok(())
