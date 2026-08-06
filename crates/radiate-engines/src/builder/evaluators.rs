use crate::GeneticEngineBuilder;
use radiate_core::{Chromosome, Evaluator, Executor, FitnessEvaluator, ThreadSync};
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct ExecutorParams {
    pub changed: bool,
    pub executor: Arc<Executor>,
}

#[derive(Clone)]
pub struct EvaluationParams<C, T>
where
    C: Chromosome,
    T: Clone,
{
    pub evaluator: Arc<dyn Evaluator<C, T>>,
    pub fitness_executor: ExecutorParams,
    pub species_executor: ExecutorParams,
    pub event_bus_executor: ExecutorParams,
    pub sync: ThreadSync,
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

    pub fn parallel(self) -> Self {
        #[cfg(feature = "rayon")]
        {
            self.executor(Executor::WorkerPool)
        }
        #[cfg(not(feature = "rayon"))]
        {
            let num_cpus = std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4);
            self.executor(Executor::FixedSizedWorkerPool(num_cpus))
        }
    }

    pub fn executor(mut self, executor: Executor) -> Self {
        let executor = Arc::new(executor);
        let param = ExecutorParams {
            changed: true,
            executor: executor.clone(),
        };
        self.params.evaluation_params = EvaluationParams {
            evaluator: Arc::new(FitnessEvaluator::new(executor.clone())),
            fitness_executor: param.clone(),
            species_executor: param.clone(),
            event_bus_executor: param,
            sync: self.params.evaluation_params.sync.clone(),
        };
        self
    }

    pub fn fitness_executor(mut self, executor: impl Into<Arc<Executor>>) -> Self {
        self.params.evaluation_params.fitness_executor = ExecutorParams {
            changed: true,
            executor: executor.into(),
        };
        self
    }

    pub fn species_executor(mut self, executor: impl Into<Arc<Executor>>) -> Self {
        self.params.evaluation_params.species_executor = ExecutorParams {
            changed: true,
            executor: executor.into(),
        };
        self
    }

    pub fn broker_executor(mut self, executor: impl Into<Arc<Executor>>) -> Self {
        self.params.evaluation_params.event_bus_executor = ExecutorParams {
            changed: true,
            executor: executor.into(),
        };
        self
    }
}
