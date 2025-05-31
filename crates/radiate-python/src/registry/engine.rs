use crate::FreeThreadPyEvaluator;
use radiate::{Chromosome, Epoch, GeneticEngineBuilder, steps::SequentialEvaluator};

pub fn set_evaluator<C, T, E>(
    builder: GeneticEngineBuilder<C, T, E>,
    num_threads: &usize,
) -> GeneticEngineBuilder<C, T, E>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send + Sync,
    E: Epoch<Chromosome = C>,
{
    match num_threads {
        1 => builder.executor(SequentialEvaluator::new()),
        _ => builder
            .num_threads(*num_threads)
            .executor(FreeThreadPyEvaluator::new(*num_threads)),
    }
}
