use radiate_core::{
    Chromosome, Ecosystem, MetricSet, Objective, Problem, engine::EngineStep, metric_names,
    thread_pool::ThreadPool,
};
use std::sync::Arc;

pub struct EvaluateStep<C: Chromosome, T> {
    pub(crate) objective: Objective,
    pub(crate) thread_pool: Arc<ThreadPool>,
    pub(crate) problem: Arc<dyn Problem<C, T>>,
}

impl<C: Chromosome, T> EvaluateStep<C, T> {
    pub fn new(
        objective: Objective,
        thread_pool: Arc<ThreadPool>,
        problem: Arc<dyn Problem<C, T>>,
    ) -> Self {
        EvaluateStep {
            objective,
            thread_pool,
            problem,
        }
    }
}

impl<C, T> EngineStep<C> for EvaluateStep<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    fn execute(&mut self, _: usize, metrics: &mut MetricSet, ecosystem: &mut Ecosystem<C>) {
        let timer = std::time::Instant::now();

        let mut work_results = Vec::new();
        for idx in 0..ecosystem.population.len() {
            let individual = &mut ecosystem.population[idx];
            if individual.score().is_some() {
                continue;
            } else {
                let problem = Arc::clone(&self.problem);
                let geno = individual.take_genotype();
                let work = self.thread_pool.submit_with_result(move || {
                    let score = problem.eval(&geno);
                    (idx, score, geno)
                });

                work_results.push(work);
            }
        }

        let count = work_results.len() as f32;
        for work_result in work_results {
            let (idx, score, genotype) = work_result.result();
            ecosystem.population[idx].set_score(Some(score));
            ecosystem.population[idx].set_genotype(genotype);
        }

        self.objective.sort(&mut ecosystem.population);

        metrics.upsert_operations(metric_names::FITNESS, count, timer.elapsed());
    }
}
