use super::EngineStep;
use crate::domain::{thread_pool::WaitGroup, timer::Timer};
use crate::thread_pool::ThreadPool;
use crate::{Chromosome, EngineContext, GeneticEngineParams, Objective, Phenotype, sync::RwCell};
use crate::{Front, metric_names};
use std::sync::Arc;

pub struct FrontStep<C: Chromosome> {
    front: RwCell<Front<Phenotype<C>>>,
    dominates_buffer: RwCell<Vec<bool>>,
    remove_buffer: RwCell<Vec<Arc<Phenotype<C>>>>,
    thread_pool: Arc<ThreadPool>,
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
        if let Objective::Single(_) = params.objective() {
            return None;
        }

        let dominates_buffer = RwCell::new(vec![false; params.population().len()]);
        let remove_buffer = RwCell::new(Vec::new());
        let front = params.front().clone();

        Some(Box::new(FrontStep {
            front: RwCell::new(front),
            dominates_buffer: dominates_buffer,
            remove_buffer: remove_buffer,
            thread_pool: Arc::clone(&params.thread_pool()),
        }))
    }

    /// Updates the front of the population using the scores of the individuals. The front is a collection
    /// of individuals that are not dominated by any other individual in the population. This method is only
    /// called if the objective is multi-objective, as the front is not relevant for single-objective optimization.
    /// The front is updated in a separate thread to avoid blocking the main thread while the front is being calculated.
    /// This can significantly speed up the calculation of the front for large populations.
    fn execute(&self, ctx: &mut EngineContext<C, T>) {
        let timer = Timer::new();
        let wg = WaitGroup::new();

        let new_individuals = ctx
            .population
            .iter()
            .filter(|pheno| pheno.generation() == ctx.index)
            .collect::<Vec<&Phenotype<C>>>();

        self.dominates_buffer.write().fill(false);
        self.remove_buffer.write().clear();

        for (idx, member) in new_individuals.iter().enumerate() {
            let pheno = Phenotype::clone(member);
            let front_clone = RwCell::clone(&self.front);
            let doms_vector = RwCell::clone(&self.dominates_buffer);
            let remove_vector = RwCell::clone(&self.remove_buffer);

            self.thread_pool.group_submit(&wg, move || {
                let (dominates, to_remove) = front_clone.read().dominates(&pheno);

                if dominates {
                    doms_vector.write().get_mut(idx).map(|v| *v = true);
                    remove_vector
                        .write()
                        .extend(to_remove.iter().map(|r| r.clone()));
                }
            });
        }

        let count = wg.wait();

        let dominates_vector = self
            .dominates_buffer
            .read()
            .iter()
            .enumerate()
            .filter(|(_, is_dominating)| **is_dominating)
            .map(|(idx, _)| new_individuals[idx])
            .collect::<Vec<&Phenotype<C>>>();
        let mut remove_vector = self.remove_buffer.write();

        remove_vector.dedup();

        self.front
            .write()
            .clean(dominates_vector, remove_vector.as_slice());

        ctx.front = self.front.read().clone();
        ctx.record_operation(metric_names::FRONT, count as f32, timer);
    }
}
