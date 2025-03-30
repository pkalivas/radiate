use super::EngineStep;
use crate::domain::thread_pool::WaitGroup;
use crate::thread_pool::ThreadPool;
use crate::{
    Chromosome, GeneticEngineParams, Metric, Objective, Phenotype, Population, Problem, Species,
    metric_names,
};
use std::sync::Arc;

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
    fn execute(
        &self,
        _: usize,
        population: &mut Population<C>,
        _: &mut Vec<Species<C>>,
    ) -> Vec<Metric> {
        let wg = WaitGroup::new();

        for pheno in population.iter() {
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

        let count = wg.wait();
        self.objective.sort(population);

        return vec![Metric::new_value(metric_names::FITNESS).with_count_value(count)];
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
