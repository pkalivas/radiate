use crate::GeneticEngineBuilder;
use radiate_core::{Chromosome, Evaluator, Executor, FitnessEvaluator};
use std::sync::Arc;

#[derive(Clone)]
pub struct EvaluationParams<C, T>
where
    C: Chromosome,
    T: Clone,
{
    pub evaluator: Arc<dyn Evaluator<C, T>>,
    pub fitness_executor: Arc<Executor>,
    pub species_executor: Arc<Executor>,
    pub bus_executor: Arc<Executor>,
}

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    pub fn evaluator<V: Evaluator<C, T> + 'static>(mut self, evaluator: V) -> Self {
        self.params.evaluation_params.evaluator = Arc::new(evaluator);
        self
    }

    #[cfg(feature = "rayon")]
    pub fn parallel(self) -> Self {
        self.executor(Executor::WorkerPool)
    }

    pub fn executor(mut self, executor: Executor) -> Self {
        let executor = Arc::new(executor);
        self.params.evaluation_params = EvaluationParams {
            evaluator: Arc::new(FitnessEvaluator::new(executor.clone())),
            fitness_executor: executor.clone(),
            species_executor: executor.clone(),
            bus_executor: executor.clone(),
        };
        self
    }

    pub fn fitness_executor(mut self, executor: impl Into<Arc<Executor>>) -> Self {
        self.params.evaluation_params.fitness_executor = executor.into();
        self
    }

    pub fn species_executor(mut self, executor: impl Into<Arc<Executor>>) -> Self {
        self.params.evaluation_params.species_executor = executor.into();
        self
    }

    pub fn bus_executor(mut self, executor: impl Into<Arc<Executor>>) -> Self {
        self.params.evaluation_params.bus_executor = executor.into();
        self
    }
}
