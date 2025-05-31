use super::ComponentRegistry;
use crate::{FreeThreadPyEvaluator, PyEngineBuilder};
use radiate::{Chromosome, Epoch, Executor, GeneticEngineBuilder};
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct EvaluatorRegistry;

impl EvaluatorRegistry {
    pub fn new() -> Self {
        EvaluatorRegistry
    }
}

impl ComponentRegistry for EvaluatorRegistry {
    fn apply<C, T, E>(
        &self,
        engine_builder: GeneticEngineBuilder<C, T, E>,
        py_builder: &PyEngineBuilder,
        _: crate::PyGeneType,
    ) -> GeneticEngineBuilder<C, T, E>
    where
        C: Chromosome + Clone + PartialEq + 'static,
        T: Clone + Send + Sync + 'static,
        E: Epoch<Chromosome = C> + 'static,
    {
        match py_builder.num_threads {
            1 => engine_builder.evaluator(FreeThreadPyEvaluator::new(Arc::new(Executor::Serial))),
            n => {
                let executor = Arc::new(Executor::worker_pool(n));
                engine_builder
                    .executor(executor.clone())
                    .evaluator(FreeThreadPyEvaluator::new(executor))
            }
        }
    }
}
