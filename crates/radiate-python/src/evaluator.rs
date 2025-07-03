use crate::{
    IntoPyObjectValue,
    problem::{PyProblem, call_fitness},
};
use pyo3::{IntoPyObjectExt, Py, PyAny, Python};
use radiate::{Chromosome, Ecosystem, Evaluator, Executor, Problem};
use std::sync::Arc;

/// Simple wrapper around the batch that will be evaluated
/// together on the same thread or in the same scope
struct PyBatch {
    indices: Vec<usize>,
    genotypes: Vec<Py<PyAny>>,
    fitness_fn: Py<PyAny>,
}

/// Based off of the pyo3 docuntation: https://pyo3.rs/v0.24.2/parallelism
///
/// The [PyEvaluator] is an [Evaluator<C, T>] implementation that allows for free-threaded evaluation.
/// We avoid Python's GIL by using the `allow_threads` method, bypassing the
/// GIL for the duration of the evaluation.
pub struct FreeThreadPyEvaluator<C: Chromosome, T: IntoPyObjectValue> {
    executor: Executor,
    problem: PyProblem<C, T>,
}

impl<C: Chromosome, T: IntoPyObjectValue> FreeThreadPyEvaluator<C, T> {
    pub fn new(executor: Executor, problem: PyProblem<C, T>) -> Self {
        FreeThreadPyEvaluator { executor, problem }
    }
}

impl<C, T> Evaluator<C, T> for FreeThreadPyEvaluator<C, T>
where
    C: Chromosome + 'static,
    T: IntoPyObjectValue + Send + Sync + 'static,
{
    fn eval(&self, ecosystem: &mut Ecosystem<C>, _: Arc<dyn Problem<C, T>>) -> usize {
        Python::with_gil(|outer| {
            let mut jobs = Vec::new();
            let len = ecosystem.population.len();
            for idx in 0..len {
                if ecosystem.population[idx].score().is_none() {
                    let geno = self
                        .problem
                        .decode_with_py(outer, ecosystem.population[idx].genotype())
                        .into_py(outer);
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
                    genotypes: batch
                        .into_iter()
                        .map(|(_, geno)| geno.into_py_any(outer).unwrap())
                        .collect::<Vec<_>>(),
                    fitness_fn: self.problem.fitness_func().clone_ref(outer),
                };

                batches.push(batch);
            }

            outer.allow_threads(|| {
                let jobs = batches
                    .into_iter()
                    .map(|batch| {
                        move || {
                            Python::with_gil(|inner| {
                                let problem = batch.fitness_fn.bind_borrowed(inner);
                                let mut scores = Vec::with_capacity(batch.genotypes.len());
                                for ind in batch.genotypes.iter() {
                                    let borrowed = ind.bind_borrowed(inner);
                                    let score = call_fitness(inner, problem, borrowed);
                                    scores.push(score);
                                }

                                (batch, scores)
                            })
                        }
                    })
                    .collect::<Vec<_>>();

                let results = self.executor.execute_batch(jobs);

                let mut count = 0;
                for (batch, scores) in results {
                    for ((idx, _), score) in batch
                        .indices
                        .iter()
                        .zip(batch.genotypes.into_iter())
                        .zip(scores.into_iter())
                    {
                        count += 1;
                        ecosystem.population[*idx].set_score(Some(score));
                    }
                }

                count
            })
        })
    }
}
