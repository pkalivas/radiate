use radiate_core::{
    Chromosome, Ecosystem, Executor, MetricSet, Objective, Problem, SerialExecutor,
    WorkerPoolExecutor, engine::EngineStep, metric_names,
};
use std::sync::Arc;

pub trait Evaluator<C: Chromosome, T>: Send + Sync {
    fn eval(&self, ecosystem: &mut Ecosystem<C>, problem: Arc<dyn Problem<C, T>>) -> usize;
}

#[derive(Clone, Default)]
pub struct SequentialEvaluator {
    executor: SerialExecutor,
}

impl SequentialEvaluator {
    pub fn new() -> Self {
        Self {
            executor: SerialExecutor::new(),
        }
    }
}

impl<C: Chromosome, T> Evaluator<C, T> for SequentialEvaluator
where
    T: 'static,
    C: Chromosome + 'static,
{
    fn eval(&self, ecosystem: &mut Ecosystem<C>, problem: Arc<dyn Problem<C, T>>) -> usize {
        let mut jobs = Vec::new();
        let len = ecosystem.population.len();
        for idx in 0..len {
            if ecosystem.population[idx].score().is_none() {
                let geno = ecosystem.population[idx].take_genotype();
                jobs.push((idx, geno));
            }
        }

        let results = self.executor.execute_batch(
            jobs.into_iter()
                .map(|(idx, geno)| {
                    let problem = Arc::clone(&problem);
                    move || {
                        let score = problem.eval(&geno);
                        (idx, score, geno)
                    }
                })
                .collect::<Vec<_>>(),
        );

        let count = results.len();
        for (idx, score, genotype) in results {
            ecosystem.population[idx].set_score(Some(score));
            ecosystem.population[idx].set_genotype(genotype);
        }

        count
    }
}

pub struct WorkerPoolEvaluator {
    executor: WorkerPoolExecutor,
}

impl WorkerPoolEvaluator {
    pub fn new(num_threads: usize) -> Self {
        Self {
            executor: WorkerPoolExecutor::new(num_threads),
        }
    }
}

impl<C: Chromosome, T> Evaluator<C, T> for WorkerPoolEvaluator
where
    C: Chromosome + 'static,
    T: Send + Sync + 'static,
{
    fn eval(&self, ecosystem: &mut Ecosystem<C>, problem: Arc<dyn Problem<C, T>>) -> usize {
        let mut jobs = Vec::new();
        let len = ecosystem.population.len();
        for idx in 0..len {
            if ecosystem.population[idx].score().is_none() {
                let geno = ecosystem.population[idx].take_genotype();
                jobs.push((idx, geno));
            }
        }

        let results = self.executor.execute_batch(
            jobs.into_iter()
                .map(|(idx, geno)| {
                    let problem = Arc::clone(&problem);
                    move || {
                        let score = problem.eval(&geno);
                        (idx, score, geno)
                    }
                })
                .collect::<Vec<_>>(),
        );

        let count = results.len();
        for result in results {
            let (idx, score, genotype) = result;
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
}

impl<C: Chromosome, T> EvaluateStep<C, T> {
    pub fn new(
        objective: Objective,
        problem: Arc<dyn Problem<C, T>>,
        evaluator: Arc<dyn Evaluator<C, T>>,
    ) -> Self {
        EvaluateStep {
            objective,
            evaluator,
            problem,
        }
    }
}

impl<C, T> EngineStep<C> for EvaluateStep<C, T>
where
    C: Chromosome + PartialEq + 'static,
{
    fn execute(&mut self, _: usize, metrics: &mut MetricSet, ecosystem: &mut Ecosystem<C>) {
        let timer = std::time::Instant::now();

        let count = self.evaluator.eval(ecosystem, Arc::clone(&self.problem)) as f32;

        self.objective.sort(&mut ecosystem.population);

        metrics.upsert_operations(metric_names::FITNESS, count, timer.elapsed());
    }
}
