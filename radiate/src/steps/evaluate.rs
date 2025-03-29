use std::sync::Arc;

use super::EngineStep;
use crate::domain::{thread_pool::WaitGroup, timer::Timer};
use crate::thread_pool::ThreadPool;
use crate::{Chromosome, EngineContext, GeneticEngineParams, Objective, Phenotype, Problem};

pub struct EvaluateStep<C: Chromosome, T> {
    objective: Objective,
    thread_pool: Arc<ThreadPool>,
    problem: Arc<dyn Problem<C, T>>,
}

/// Evaluates the fitness of each individual in the population using the fitness function
/// provided in the genetic engine parameters. The score is then used to rank the individuals
/// in the population.
///
/// Importantly, this method uses a thread pool to evaluate the fitness of each individual in
/// parallel, which can significantly speed up the evaluation process for large populations.
/// It will also only evaluate individuals that have not yet been scored, which saves time
/// by avoiding redundant evaluations.
impl<C, T> EngineStep<C, T> for EvaluateStep<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    fn execute(&self, ctx: &mut EngineContext<C, T>) {
        let timer = Timer::new();
        let wg = WaitGroup::new();

        for pheno in ctx.population.iter() {
            if pheno.score().is_some() {
                continue;
            } else {
                let problem = Arc::clone(&self.problem);
                let mut member = Phenotype::clone(pheno);

                self.thread_pool.group_submit(&wg, move || {
                    let score = problem.eval(&member.genotype());
                    member.set_score(Some(score));
                });
            }
        }

        let count = wg.wait() as f32;
        ctx.record_operation("EvaluateStep", count, timer);

        self.objective.sort(&mut ctx.population);
    }

    fn register(params: &GeneticEngineParams<C, T>) -> Option<Box<Self>>
    where
        Self: Sized,
    {
        Some(Box::new(EvaluateStep {
            objective: params.objective().clone(),
            thread_pool: Arc::clone(params.thread_pool()),
            problem: Arc::clone(&params.problem()),
        }))
    }
}
