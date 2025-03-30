use super::EngineStep;
use crate::domain::timer::Timer;
use crate::thread_pool::{Scope, SingleFlag, ThreadPool};
use crate::{Chromosome, EngineContext, GeneticEngineParams, Objective, Phenotype, sync::RwCell};
use crate::{Front, TimeStatistic, metric_names};
use std::sync::{Arc, Mutex};

pub struct FrontStep<C: Chromosome> {
    update_guard: SingleFlag,
    front: RwCell<Front<Phenotype<C>>>,
    dominates: RwCell<Vec<bool>>,
    to_remove: RwCell<Vec<Arc<Phenotype<C>>>>,
    thread_pool: Arc<ThreadPool>,
    metric: Arc<Mutex<TimeStatistic>>,
}

impl<C, T> EngineStep<C, T> for FrontStep<C>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    fn register(params: &GeneticEngineParams<C, T>) -> Option<Box<Self>>
    where
        Self: Sized,
    {
        if matches!(params.objective(), Objective::Single(_)) {
            return None;
        }

        Some(Box::new(FrontStep {
            update_guard: SingleFlag::new(),
            front: RwCell::clone(&params.front()),
            dominates: RwCell::new(vec![false; params.population().len()]),
            to_remove: RwCell::new(Vec::new()),
            thread_pool: Arc::clone(&params.thread_pool()),
            metric: Arc::new(Mutex::new(TimeStatistic::default())),
        }))
    }

    /// Updates the front of the population using the scores of the individuals. The front is a collection
    /// of individuals that are not dominated by any other individual in the population. This method is only
    /// called if the objective is multi-objective, as the front is not relevant for single-objective optimization.
    /// The front is updated in a separate thread to avoid blocking the main thread while the front is being calculated.
    /// This can significantly speed up the calculation of the front for large populations.
    fn execute(&self, ctx: &mut EngineContext<C, T>) {
        self.update_guard.wait();
        self.update_guard.start();

        let flag = self.update_guard.clone();

        self.dominates.write().fill(false);
        self.to_remove.write().clear();

        let metric_clone = Arc::clone(&self.metric);
        let update = FrontUpdate::new(
            ctx.new_members(),
            &self.front,
            &self.dominates,
            &self.to_remove,
        );

        self.thread_pool.submit_scoped(move |scope| {
            let timer = Timer::new();

            update.spawn_tasks(scope);
            update.finalize_front();

            metric_clone.lock().unwrap().add(timer.duration());
            flag.finish();
        });

        let count = self.dominates.read().iter().filter(|flag| **flag).count();
        ctx.record_operation(
            metric_names::FRONT,
            count as f32,
            self.metric.lock().unwrap().last_time(),
        );
    }
}

struct FrontUpdate<C: Chromosome> {
    candidates: Vec<Phenotype<C>>,
    front: RwCell<Front<Phenotype<C>>>,
    dominates: RwCell<Vec<bool>>,
    to_remove: RwCell<Vec<Arc<Phenotype<C>>>>,
}

impl<C: Chromosome> FrontUpdate<C>
where
    C: Chromosome + 'static,
{
    pub fn new(
        candidates: Vec<Phenotype<C>>,
        front: &RwCell<Front<Phenotype<C>>>,
        dominates: &RwCell<Vec<bool>>,
        to_remove: &RwCell<Vec<Arc<Phenotype<C>>>>,
    ) -> Self {
        FrontUpdate {
            candidates,
            front: RwCell::clone(front),
            dominates: RwCell::clone(dominates),
            to_remove: RwCell::clone(to_remove),
        }
    }
    pub fn spawn_tasks(&self, scope: Scope) {
        for (idx, pheno) in self.candidates.iter().enumerate() {
            let pheno = Phenotype::clone(pheno);
            let front = RwCell::clone(&self.front);
            let dominates = RwCell::clone(&self.dominates);
            let to_remove = RwCell::clone(&self.to_remove);

            scope.spawn(move || {
                let (dom, to_rm) = front.read().dominates(&pheno);
                if dom {
                    dominates.write().get_mut(idx).map(|v| *v = true);
                    to_remove.write().extend(to_rm.iter().cloned());
                }
            });
        }
    }

    fn finalize_front(&self) {
        let dominators = self
            .dominates
            .read()
            .iter()
            .enumerate()
            .filter_map(|(i, &flag)| {
                if flag {
                    Some(&self.candidates[i])
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        let mut remove = self.to_remove.write();
        remove.dedup();

        self.front.write().clean(dominators, &remove);
    }
}
