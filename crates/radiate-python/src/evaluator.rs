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

        Python::with_gil(|outer| {
            outer.allow_threads(|| {
                let work_results = jobs
                    .into_iter()
                    .map(|(idx, geno)| {
                        let problem = Arc::clone(&problem);
                        thread_pool.submit_with_result(move || {
                            let score = problem.eval(&geno);
                            (idx, score, geno)
                        })
                    })
                    .collect::<Vec<_>>();

                let count = work_results.len();
                for work_result in work_results {
                    let (idx, score, genotype) = work_result.result();
                    ecosystem.population[idx].set_score(Some(score));
                    ecosystem.population[idx].set_genotype(genotype);
                }

                count
            })
        })
    }
}
