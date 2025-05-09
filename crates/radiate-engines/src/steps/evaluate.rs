use radiate_core::{
    Chromosome, Ecosystem, MetricSet, Objective, Problem, engine::EngineStep, metric_names,
    thread_pool::ThreadPool,
};
use std::sync::Arc;

pub trait Evaluator<C: Chromosome, T>: Send + Sync {
    fn eval(
        &self,
        ecosystem: &mut Ecosystem<C>,
        thread_pool: Arc<ThreadPool>,
        problem: Arc<dyn Problem<C, T>>,
    ) -> usize;
}

pub struct SequentialEvaluator;

impl<C: Chromosome, T> Evaluator<C, T> for SequentialEvaluator
where
    C: Chromosome + 'static,
{
    fn eval(
        &self,
        ecosystem: &mut Ecosystem<C>,
        _thread_pool: Arc<ThreadPool>,
        problem: Arc<dyn Problem<C, T>>,
    ) -> usize {
        let mut count = 0;
        for individual in ecosystem.population.iter_mut() {
            if individual.score().is_some() {
                continue;
            } else {
                let geno = individual.take_genotype();
                let score = problem.eval(&geno);
                individual.set_score(Some(score));
                individual.set_genotype(geno);
                count += 1;
            }
        }

        count
    }
}

pub struct WorkerPoolEvaluator;

impl<C: Chromosome, T> Evaluator<C, T> for WorkerPoolEvaluator
where
    C: Chromosome + 'static,
    T: Send + Sync + 'static,
{
    fn eval(
        &self,
        ecosystem: &mut Ecosystem<C>,
        thread_pool: Arc<ThreadPool>,
        problem: Arc<dyn Problem<C, T>>,
    ) -> usize {
        let mut jobs = Vec::new();
        let len = ecosystem.population.len();
        for idx in 0..len {
            if ecosystem.population[idx].score().is_none() {
                let geno = ecosystem.population[idx].take_genotype();
                jobs.push((idx, geno));
            }
        }

        let work_results = jobs
            .into_iter()
            .map(|(idx, geno)| {
                let problem = Arc::clone(&problem);
                thread_pool.submit_with_result(move || {
                    let score = problem.eval(&geno);
                    (idx, score, geno)
                })
            })
            .collect::<Vec<_>>();

        let count = work_results.len();
        for work_result in work_results {
            let (idx, score, genotype) = work_result.result();
            ecosystem.population[idx].set_score(Some(score));
            ecosystem.population[idx].set_genotype(genotype);
        }

        count
    }
}

pub struct EvaluateStep<C: Chromosome, T> {
    pub(crate) objective: Objective,
    pub(crate) evaluator: Arc<dyn Evaluator<C, T>>,
    pub(crate) problem: Arc<dyn Problem<C, T>>,
    pub(crate) thread_pool: Arc<ThreadPool>,
}

impl<C: Chromosome, T> EvaluateStep<C, T> {
    pub fn new(
        objective: Objective,
        thread_pool: Arc<ThreadPool>,
        problem: Arc<dyn Problem<C, T>>,
        evaluator: Arc<dyn Evaluator<C, T>>,
    ) -> Self {
        EvaluateStep {
            objective,
            evaluator,
            problem,
            thread_pool,
        }
    }
}

impl<C, T> EngineStep<C> for EvaluateStep<C, T>
where
    C: Chromosome + 'static,
{
    fn execute(&mut self, _: usize, metrics: &mut MetricSet, ecosystem: &mut Ecosystem<C>) {
        let timer = std::time::Instant::now();

        let count = self.evaluator.eval(
            ecosystem,
            Arc::clone(&self.thread_pool),
            Arc::clone(&self.problem),
        ) as f32;

        self.objective.sort(&mut ecosystem.population);

        metrics.upsert_operations(metric_names::FITNESS, count, timer.elapsed());
    }
}
