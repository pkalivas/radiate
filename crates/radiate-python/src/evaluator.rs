use pyo3::Python;
use radiate::{Chromosome, Ecosystem, Problem, steps::Evaluator, thread_pool::ThreadPool};
use std::sync::Arc;

/// Based off of the pyo3 docuntation: https://pyo3.rs/v0.24.2/parallelism
///
/// The [PyEvaluator] is an [Evaluator<C, T>] implementation that allows for free-threaded evaluation.
/// We avoid Python's GIL by using the `allow_threads` method, bypassing the
/// GIL for the duration of the evaluation.
#[derive(Default)]
pub struct PyEvaluator;

impl<C: Chromosome, T> Evaluator<C, T> for PyEvaluator
where
    C: Chromosome + 'static,
    T: Send + Sync + 'static,
{
    fn eval(
        &self,
        ecosystem: &mut Ecosystem<C>,
        thread_pool: Arc<ThreadPool>,
        problem: Arc<dyn Problem<C, T>>,
    ) -> usize {
        let mut jobs = Vec::new();
        let len = ecosystem.population.len();
        for idx in 0..len {
            if ecosystem.population[idx].score().is_none() {
                let geno = ecosystem.population[idx].take_genotype();
                jobs.push((idx, geno));
            }
        }

        let num_workers = thread_pool.num_workers();
        let batch_size = (jobs.len() + num_workers - 1) / num_workers;

        if batch_size == 0 {
            return 0;
        }

        let mut batches = Vec::new();
        for i in (0..jobs.len()).step_by(batch_size) {
            let end = std::cmp::min(i + batch_size, jobs.len());
            let batch = jobs[i..end].to_vec();
            batches.push((
                batch.iter().map(|(idx, _)| *idx).collect::<Vec<_>>(),
                batch.into_iter().map(|(_, geno)| geno).collect::<Vec<_>>(),
            ));
        }

        Python::with_gil(|outer| {
            outer.allow_threads(|| {
                let work_results = batches
                    .into_iter()
                    .map(|batch| {
                        let problem = Arc::clone(&problem);
                        thread_pool.submit_with_result(move || {
                            let scores = problem.eval_batch(&batch.1);
                            (batch, scores)
                        })
                    })
                    .collect::<Vec<_>>();

                let mut count = 0;
                for work_result in work_results {
                    let (batch, scores) = work_result.result();

                    for ((idx, geno), score) in batch
                        .0
                        .iter()
                        .zip(batch.1.into_iter())
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
