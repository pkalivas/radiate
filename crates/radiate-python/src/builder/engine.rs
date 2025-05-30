use crate::PyEvaluator;
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
        1 => builder.evaluator(SequentialEvaluator),
        _ => builder.num_threads(*num_threads).evaluator(PyEvaluator),
    }
}
