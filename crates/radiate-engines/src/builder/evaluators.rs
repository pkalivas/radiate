use crate::GeneticEngineBuilder;
use radiate_core::{
    Chromosome, Epoch, Evaluator, Executor, FitnessEvaluator, thread_pool::ThreadPool,
};
use radiate_error::radiate_err;
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
    pub front_executor: Arc<Executor>,
    pub bus_executor: Arc<Executor>,
}

impl<C, T, E> GeneticEngineBuilder<C, T, E>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
    E: Epoch<C>,
{
    pub fn evaluator<V: Evaluator<C, T> + 'static>(mut self, evaluator: V) -> Self {
        self.params.evaluation_params.evaluator = Arc::new(evaluator);
        self
    }

    pub fn num_threads(mut self, num_threads: usize) -> Self
    where
        T: Send + Sync + 'static,
    {
        if num_threads < 1 {
            self.errors
                .push(radiate_err!(InvalidConfig: "num_threads must be greater than 0"));
        }

        #[cfg(feature = "rayon")]
        let executor = if num_threads == 1 {
            Arc::new(Executor::Serial)
        } else {
            rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build_global()
                .map(|_| Arc::new(Executor::Rayon))
                .unwrap_or_else(|_| Arc::new(Executor::WorkerPool(ThreadPool::new(num_threads))))
        };

        #[cfg(not(feature = "rayon"))]
        let executor = if num_threads == 1 {
            Arc::new(Executor::Serial)
        } else {
            Arc::new(Executor::WorkerPool(ThreadPool::new(num_threads)))
        };

        self.params.evaluation_params = EvaluationParams {
            evaluator: Arc::new(FitnessEvaluator::new(executor.clone())),
            fitness_executor: executor.clone(),
            species_executor: executor.clone(),
            front_executor: executor.clone(),
            bus_executor: executor,
        };
        self
    }

    pub fn executor(mut self, executor: impl Into<Arc<Executor>>) -> Self {
        let executor = executor.into();
        self.params.evaluation_params = EvaluationParams {
            evaluator: Arc::new(FitnessEvaluator::new(executor.clone())),
            fitness_executor: executor.clone(),
            species_executor: executor.clone(),
            front_executor: executor.clone(),
            bus_executor: executor,
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

    pub fn front_executor(mut self, executor: impl Into<Arc<Executor>>) -> Self {
        self.params.evaluation_params.front_executor = executor.into();
        self
    }

    pub fn bus_executor(mut self, executor: impl Into<Arc<Executor>>) -> Self {
        self.params.evaluation_params.bus_executor = executor.into();
        self
    }
}
