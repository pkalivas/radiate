// use pyo3::{Borrowed, IntoPyObjectExt, Py, PyAny, Python};
// use radiate::{Chromosome, Ecosystem, Evaluator, Executor, Genotype, Problem, Score};
// use std::sync::Arc;

// use crate::{PyProblem, problem};

// /// Simple wrapper around the batch that will be evaluated
// /// together on the same thread or in the same scope
// struct PyBatch {
//     id: usize,
//     indices: Vec<usize>,
//     genotypes: Vec<Py<PyAny>>,
//     fitness_fn: Py<PyAny>,
// }

// /// Based off of the pyo3 docuntation: https://pyo3.rs/v0.24.2/parallelism
// ///
// /// The [PyEvaluator] is an [Evaluator<C, T>] implementation that allows for free-threaded evaluation.
// /// We avoid Python's GIL by using the `allow_threads` method, bypassing the
// /// GIL for the duration of the evaluation.
// pub struct FreeThreadPyEvaluator<C: Chromosome> {
//     executor: Arc<Executor>,
//     problem: Arc<PyProblem<C>>,
// }

// impl<C: Chromosome> FreeThreadPyEvaluator<C> {
//     pub fn new(executor: Arc<Executor>, problem: Arc<PyProblem<C>>) -> Self {
//         FreeThreadPyEvaluator { executor, problem }
//     }
// }

// impl<C: Chromosome, T> Evaluator<C, T> for FreeThreadPyEvaluator<C>
// where
//     C: Chromosome + 'static,
//     T: Send + Sync + 'static,
// {
//     fn eval(&self, ecosystem: &mut Ecosystem<C>, _: Arc<dyn Problem<C, T>>) -> usize {
//         // let mut jobs = Vec::new();
//         // let len = ecosystem.population.len();
//         // for idx in 0..len {
//         //     if ecosystem.population[idx].score().is_none() {
//         //         let geno = ecosystem.population[idx].take_genotype();
//         //         jobs.push((idx, Some(geno)));
//         //     }
//         // }

//         // let num_workers = self.executor.num_workers();
//         // let batch_size = (jobs.len() + num_workers - 1) / num_workers;

//         // if batch_size == 0 {
//         //     return 0;
//         // }

//         // let mut batches = Vec::new();
//         // for i in (0..jobs.len()).step_by(batch_size) {
//         //     let end = std::cmp::min(i + batch_size, jobs.len());

//         //     let batch = jobs
//         //         .iter_mut()
//         //         .skip(i)
//         //         .take(end - i)
//         //         .map(|(idx, geno)| (*idx, geno.take().unwrap()))
//         //         .collect::<Vec<_>>();

//         //     let batch = PyBatch {
//         //         id: batches.len(),
//         //         indices: batch.iter().map(|(idx, _)| *idx).collect(),
//         //         genotypes: batch.into_iter().map(|(_, geno)| geno).collect(),
//         //     };

//         //     batches.push(batch);
//         // }

//         Python::with_gil(|outer| {
//             let mut jobs = Vec::new();
//             let len = ecosystem.population.len();
//             for idx in 0..len {
//                 if ecosystem.population[idx].score().is_none() {
//                     let geno = self
//                         .problem
//                         .decode_with_py(outer, ecosystem.population[idx].genotype());
//                     jobs.push((idx, Some(geno)));
//                 }
//             }

//             let num_workers = self.executor.num_workers();
//             let batch_size = (jobs.len() + num_workers - 1) / num_workers;

//             if batch_size == 0 {
//                 return 0;
//             }
//             let mut batches = Vec::new();
//             for i in (0..jobs.len()).step_by(batch_size) {
//                 let end = std::cmp::min(i + batch_size, jobs.len());

//                 let batch = jobs
//                     .iter_mut()
//                     .skip(i)
//                     .take(end - i)
//                     .map(|(idx, geno)| (*idx, geno.take().unwrap()))
//                     .collect::<Vec<_>>();

//                 let batch = PyBatch {
//                     id: batches.len(),
//                     indices: batch.iter().map(|(idx, _)| *idx).collect(),
//                     genotypes: batch
//                         .into_iter()
//                         .map(|(_, geno)| geno.into_py_any(outer).unwrap())
//                         .collect::<Vec<_>>(),
//                     fitness_fn: self.problem.fitness_func().clone_ref(outer),
//                 };

//                 batches.push(batch);
//             }

//             // let fitness_fn = self.problem.fitness_func().clone_ref(outer);

//             outer.allow_threads(|| {
//                 let jobs = batches
//                     .into_iter()
//                     .map(|batch| {
//                         // let problem = Arc::clone(&self.problem);
//                         // // let fitness_fn = fitness_fn.clone();
//                         // let fitness_fn = fitness_fn.clone_ref(outer);

//                         move || {
//                             Python::with_gil(|inner| {
//                                 // let fitness_fn = problem.fitness_func().clone_ref(inner);
//                                 // let problem = fitness_fn.bind_borrowed(inner);
//                                 println!("Evaluating batch {}", batch.id);
//                                 let problem = batch.fitness_fn.bind_borrowed(inner);
//                                 let mut scores = Vec::with_capacity(batch.genotypes.len());
//                                 for ind in batch.genotypes.iter() {
//                                     let borrowed = ind.bind_borrowed(inner);
//                                     let score = call_borrowed(inner, problem, borrowed);
//                                     scores.push(score);
//                                 }
//                                 // let scores = batch
//                                 //     .genotypes
//                                 //     .iter()
//                                 //     .map(|ind| problem.decode_with_py(inner, ind).inner)
//                                 //     .map(|phenotype| problem.call_fitness(inner, &phenotype))
//                                 //     .collect::<Vec<Score>>();

//                                 println!("Finished evaluating batch {}", batch.id);

//                                 (batch, scores)
//                             })
//                         }
//                     })
//                     .collect::<Vec<_>>();

//                 println!("");

//                 let results = self.executor.execute_batch(jobs);

//                 let mut count = 0;
//                 for (batch, scores) in results {
//                     for ((idx, geno), score) in batch
//                         .indices
//                         .iter()
//                         .zip(batch.genotypes.into_iter())
//                         .zip(scores.into_iter())
//                     {
//                         count += 1;
//                         // ecosystem.population[*idx].set_genotype(geno);
//                         ecosystem.population[*idx].set_score(Some(score));
//                     }
//                 }

//                 count
//             })
//         })
//     }
// }

// pub fn call_borrowed<'a, 'py>(
//     py: Python<'py>,
//     func: Borrowed<'a, 'py, PyAny>,
//     input: Borrowed<'a, 'py, PyAny>,
// ) -> Score {
//     let any_value = func
//         .as_ref()
//         .call1(py, (input,))
//         .expect("Python call failed");

//     if let Ok(parsed) = any_value.extract::<f32>(py) {
//         return Score::from(parsed);
//     } else if let Ok(parsed) = any_value.extract::<i32>(py) {
//         return Score::from(parsed as f32);
//     } else if let Ok(parsed) = any_value.extract::<f64>(py) {
//         return Score::from(parsed as f32);
//     } else if let Ok(parsed) = any_value.extract::<i64>(py) {
//         return Score::from(parsed as f32);
//     } else if let Ok(scores_vec) = any_value.extract::<Vec<f32>>(py) {
//         if scores_vec.is_empty() {
//             return Score::from(0.0);
//         } else {
//             return Score::from(scores_vec);
//         }
//     } else if let Ok(scores_vec) = any_value.extract::<Vec<i32>>(py) {
//         if scores_vec.is_empty() {
//             return Score::from(0.0);
//         } else {
//             return Score::from(scores_vec.into_iter().map(|s| s as f32).collect::<Vec<_>>());
//         }
//     }

//     panic!(
//         "Failed to extract scores from Python function call. Ensure the function returns a valid score type."
//     );
// }

use pyo3::{Borrowed, IntoPyObjectExt, Py, PyAny, Python};
use radiate::{Chromosome, Ecosystem, Evaluator, Executor, Genotype, Problem, Score};
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use std::{result, sync::Arc};

use crate::{PyProblem, problem};

/// Simple wrapper around the batch that will be evaluated
/// together on the same thread or in the same scope
struct PyBatch {
    id: usize,
    indices: Vec<usize>,
    genotypes: Vec<Py<PyAny>>,
    fitness_fn: Py<PyAny>,
}

/// Based off of the pyo3 docuntation: https://pyo3.rs/v0.24.2/parallelism
///
/// The [PyEvaluator] is an [Evaluator<C, T>] implementation that allows for free-threaded evaluation.
/// We avoid Python's GIL by using the `allow_threads` method, bypassing the
/// GIL for the duration of the evaluation.
pub struct FreeThreadPyEvaluator<C: Chromosome> {
    executor: Arc<Executor>,
    problem: Arc<PyProblem<C>>,
}

impl<C: Chromosome> FreeThreadPyEvaluator<C> {
    pub fn new(executor: Arc<Executor>, problem: Arc<PyProblem<C>>) -> Self {
        FreeThreadPyEvaluator { executor, problem }
    }
}

impl<C: Chromosome, T> Evaluator<C, T> for FreeThreadPyEvaluator<C>
where
    C: Chromosome + 'static,
    T: Send + Sync + 'static,
{
    fn eval(&self, ecosystem: &mut Ecosystem<C>, _: Arc<dyn Problem<C, T>>) -> usize {
        // let mut jobs = Vec::new();
        // let len = ecosystem.population.len();
        // for idx in 0..len {
        //     if ecosystem.population[idx].score().is_none() {
        //         let geno = ecosystem.population[idx].take_genotype();
        //         jobs.push((idx, Some(geno)));
        //     }
        // }

        // let num_workers = self.executor.num_workers();
        // let batch_size = (jobs.len() + num_workers - 1) / num_workers;

        // if batch_size == 0 {
        //     return 0;
        // }

        // let mut batches = Vec::new();
        // for i in (0..jobs.len()).step_by(batch_size) {
        //     let end = std::cmp::min(i + batch_size, jobs.len());

        //     let batch = jobs
        //         .iter_mut()
        //         .skip(i)
        //         .take(end - i)
        //         .map(|(idx, geno)| (*idx, geno.take().unwrap()))
        //         .collect::<Vec<_>>();

        //     let batch = PyBatch {
        //         id: batches.len(),
        //         indices: batch.iter().map(|(idx, _)| *idx).collect(),
        //         genotypes: batch.into_iter().map(|(_, geno)| geno).collect(),
        //     };

        //     batches.push(batch);
        // }

        Python::with_gil(|outer| {
            let mut jobs = Vec::new();
            let len = ecosystem.population.len();
            for idx in 0..len {
                if ecosystem.population[idx].score().is_none() {
                    let geno = self
                        .problem
                        .decode_with_py(outer, ecosystem.population[idx].genotype());
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
                    id: batches.len(),
                    indices: batch.iter().map(|(idx, _)| *idx).collect(),
                    genotypes: batch
                        .into_iter()
                        .map(|(_, geno)| geno.into_py_any(outer).unwrap())
                        .collect::<Vec<_>>(),
                    fitness_fn: self.problem.fitness_func().clone_ref(outer),
                };

                batches.push(batch);
            }

            // let fitness_fn = self.problem.fitness_func().clone_ref(outer);
            // let mut results = Vec::new();

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
                                    let score = call_borrowed(inner, problem, borrowed);
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
                        // ecosystem.population[*idx].set_genotype(geno);
                        ecosystem.population[*idx].set_score(Some(score));
                    }
                }

                count
            })
        })
    }
}

pub fn call_borrowed<'a, 'py>(
    py: Python<'py>,
    func: Borrowed<'a, 'py, PyAny>,
    input: Borrowed<'a, 'py, PyAny>,
) -> Score {
    let any_value = func
        .as_ref()
        .call1(py, (input,))
        .expect("Python call failed");

    if let Ok(parsed) = any_value.extract::<f32>(py) {
        return Score::from(parsed);
    } else if let Ok(parsed) = any_value.extract::<i32>(py) {
        return Score::from(parsed as f32);
    } else if let Ok(parsed) = any_value.extract::<f64>(py) {
        return Score::from(parsed as f32);
    } else if let Ok(parsed) = any_value.extract::<i64>(py) {
        return Score::from(parsed as f32);
    } else if let Ok(scores_vec) = any_value.extract::<Vec<f32>>(py) {
        if scores_vec.is_empty() {
            return Score::from(0.0);
        } else {
            return Score::from(scores_vec);
        }
    } else if let Ok(scores_vec) = any_value.extract::<Vec<i32>>(py) {
        if scores_vec.is_empty() {
            return Score::from(0.0);
        } else {
            return Score::from(scores_vec.into_iter().map(|s| s as f32).collect::<Vec<_>>());
        }
    }

    panic!(
        "Failed to extract scores from Python function call. Ensure the function returns a valid score type."
    );
}

// batches
//     .into_par_iter()
//     .map(|batch| {
//         Python::with_gil(|inner| {
//             // let fitness_fn = problem.fitness_func().clone_ref(inner);
//             // let problem = fitness_fn.bind_borrowed(inner);
//             println!("Evaluating batch {}", batch.id);
//             let problem = batch.fitness_fn.bind_borrowed(inner);
//             let mut scores = Vec::with_capacity(batch.genotypes.len());
//             for ind in batch.genotypes.iter() {
//                 let borrowed = ind.bind_borrowed(inner);
//                 let score = call_borrowed(inner, problem, borrowed);
//                 scores.push(score);
//             }
//             // let scores = batch
//             //     .genotypes
//             //     .iter()
//             //     .map(|ind| problem.decode_with_py(inner, ind).inner)
//             //     .map(|phenotype| problem.call_fitness(inner, &phenotype))
//             //     .collect::<Vec<Score>>();

//             println!("Finished evaluating batch {}", batch.id);

//             (batch, scores)
//         })
//     })
//     .collect_into_vec(&mut results);
