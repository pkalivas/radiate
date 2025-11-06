use crate::IntoPyAnyObject;
use pyo3::Python;
use radiate::{Chromosome, Ecosystem, Evaluator, Executor, Problem};
use std::sync::Arc;

/// Based off of the [pyo3 documentation](https://pyo3.rs/v0.27.1/parallelism)
///
/// The `PyEvaluator` is an [Evaluator<C, T>] implementation that allows for free-threaded evaluation.
/// This is almost the exact same as the [BatchFitnessEvaluator](radiate::evaluator::BatchFitnessEvaluator),
/// with the only difference being the adaptation to work with Python's GIL (or lack thereof).
pub struct FreeThreadPyEvaluator {
    executor: Executor,
}

impl FreeThreadPyEvaluator {
    pub fn new(executor: Executor) -> Self {
        FreeThreadPyEvaluator { executor }
    }
}

impl<C, T> Evaluator<C, T> for FreeThreadPyEvaluator
where
    C: Chromosome + 'static,
    T: IntoPyAnyObject + Send + Sync + 'static,
{
    fn eval(
        &self,
        ecosystem: &mut Ecosystem<C>,
        prob: Arc<dyn Problem<C, T>>,
    ) -> radiate::Result<usize> {
        let mut pairs = Vec::new();
        let len = ecosystem.population.len();
        for idx in 0..len {
            if ecosystem.population[idx].score().is_none() {
                let geno = ecosystem.population[idx].take_genotype()?;
                pairs.push((idx, geno));
            }
        }

        let num_workers = self.executor.num_workers();
        let batch_size = (pairs.len() + num_workers - 1) / num_workers;

        if pairs.is_empty() || batch_size == 0 {
            return Ok(0);
        }

        let mut batches = Vec::with_capacity(num_workers);

        while !pairs.is_empty() {
            let take = pairs.len().min(batch_size);

            let mut batch_indices = Vec::with_capacity(take);
            let mut batch_genotypes = Vec::with_capacity(take);

            for (idx, geno) in pairs.drain(pairs.len() - take..) {
                batch_indices.push(idx);
                batch_genotypes.push(geno);
            }

            batches.push((batch_indices, batch_genotypes));
        }

        let results = Python::attach(|py| {
            py.detach(|| {
                self.executor.execute_batch(
                    batches
                        .into_iter()
                        .map(|batch| {
                            let problem = Arc::clone(&prob);
                            move || {
                                let scores = problem.eval_batch(&batch.1);
                                (batch.0, scores, batch.1)
                            }
                        })
                        .collect(),
                )
            })
        });

        let mut count = 0;
        for (indices, scores, genotypes) in results {
            count += indices.len();
            let score_genotype_iter = scores?.into_iter().zip(genotypes.into_iter());
            for (i, (score, genotype)) in score_genotype_iter.enumerate() {
                let idx = indices[i];
                ecosystem.population[idx].set_score(Some(score));
                ecosystem.population[idx].set_genotype(genotype);
            }
        }

        Ok(count)
    }
}
