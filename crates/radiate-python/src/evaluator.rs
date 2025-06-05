use pyo3::Python;
use radiate::{Chromosome, Ecosystem, Executor, Genotype, Problem, steps::Evaluator};
use std::sync::Arc;

/// Simple wrapper around the batch that will be evaluated
/// together on the same thread or in the same scope
struct PyBatch<C: Chromosome> {
    indices: Vec<usize>,
    genotypes: Vec<Genotype<C>>,
}

/// Based off of the pyo3 docuntation: https://pyo3.rs/v0.24.2/parallelism
///
/// The [PyEvaluator] is an [Evaluator<C, T>] implementation that allows for free-threaded evaluation.
/// We avoid Python's GIL by using the `allow_threads` method, bypassing the
/// GIL for the duration of the evaluation.
pub struct FreeThreadPyEvaluator {
    executor: Arc<Executor>,
}

impl FreeThreadPyEvaluator {
    pub fn new(executor: Arc<Executor>) -> Self {
        FreeThreadPyEvaluator { executor }
    }
}

impl<C: Chromosome, T> Evaluator<C, T> for FreeThreadPyEvaluator
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
                jobs.push((idx, Some(geno)));
            }
        }

        let num_workers = self.executor.num_workers();
        let batch_size = (jobs.len() + num_workers - 1) / num_workers;

        if batch_size == 0 {
            return 0;
        }

        let mut batches = Vec::new();
        for i in (0..jobs.len()).step_by(batch_size) {
            let end = std::cmp::min(i + batch_size, jobs.len());

            let batch = jobs
                .iter_mut()
                .skip(i)
                .take(end - i)
                .map(|(idx, geno)| (*idx, geno.take().unwrap()))
                .collect::<Vec<_>>();

            let batch = PyBatch {
                indices: batch.iter().map(|(idx, _)| *idx).collect(),
                genotypes: batch.into_iter().map(|(_, geno)| geno).collect(),
            };

            batches.push(batch);
        }

        Python::with_gil(|outer| {
            outer.allow_threads(|| {
                let jobs = batches
                    .into_iter()
                    .map(|batch| {
                        let problem = Arc::clone(&problem);

                        move || {
                            let scores = problem.eval_batch(&batch.genotypes);
                            (batch, scores)
                        }
                    })
                    .collect::<Vec<_>>();

                let results = self.executor.execute_batch(jobs);

                let mut count = 0;
                for (batch, scores) in results {
                    for ((idx, geno), score) in batch
                        .indices
                        .iter()
                        .zip(batch.genotypes.into_iter())
                        .zip(scores.into_iter())
                    {
                        count += 1;
                        ecosystem.population[*idx].set_genotype(geno);
                        ecosystem.population[*idx].set_score(Some(score));
                    }
                }

                count
            })
        })
    }
}
