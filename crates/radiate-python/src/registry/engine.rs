use std::sync::Arc;

use crate::{FreeThreadPyEvaluator, PyEngineBuilder};
use radiate::{Chromosome, Epoch, Executor, GeneticEngineBuilder, steps::SequentialEvaluator};

pub fn set_evaluator<C, T, E>(
    builder: GeneticEngineBuilder<C, T, E>,
    py_builder: &PyEngineBuilder,
) -> GeneticEngineBuilder<C, T, E>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send + Sync,
    E: Epoch<Chromosome = C>,
{
    match py_builder.num_threads {
        1 => builder.evaluator(SequentialEvaluator::new()),
        n => {
            let executor = Arc::new(Executor::worker_pool(n));
            builder.executor(executor.clone())
            // .evaluator(FreeThreadPyEvaluator::new(executor))
        }
    }
}
