use pyo3::Python;
use radiate::{Chromosome, Ecosystem, Executor, Problem, steps::Evaluator};
use std::sync::Arc;

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

            batches.push((
                batch.iter().map(|(idx, _)| *idx).collect::<Vec<_>>(),
                batch.into_iter().map(|(_, geno)| geno).collect::<Vec<_>>(),
            ));
        }

        Python::with_gil(|outer| {
            outer.allow_threads(|| {
                let jobs = batches
                    .into_iter()
                    .map(|batch| {
                        let problem = Arc::clone(&problem);

                        move || {
                            let scores = problem.eval_batch(&batch.1);
                            (batch, scores)
                        }
                    })
                    .collect::<Vec<_>>();

                let results = self.executor.execute_batch(jobs);

                let mut count = 0;
                for (batch, scores) in results {
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

// In crates/radiate-python/src/executors.rs

// use pyo3::prelude::*;
// use pyo3::types::PyDict;
// use radiate_core::Executor;
// use std::sync::{Arc, Mutex, mpsc};
// use std::thread;

// /// A Python executor that uses multiprocessing to avoid GIL issues
// pub struct PythonExecutor {
//     num_processes: usize,
//     process_pool: Arc<Mutex<Vec<PyObject>>>,  // Store Python process objects
//     gil: Py<PyAny>,  // Store GIL token for Python interop
// }

// impl PythonExecutor {
//     pub fn new(num_processes: usize) -> PyResult<Self> {
//         Python::with_gil(|py| {
//             // Import multiprocessing module
//             let mp = py.import("multiprocessing")?;

//             // Create a pool of processes
//             let pool = mp.getattr("Pool")?.call1((num_processes,))?;

//             // Store the pool and GIL token
//             Ok(Self {
//                 num_processes,
//                 process_pool: Arc::new(Mutex::new(vec![pool.into()])),
//                 gil: Python::acquire_gil().into_py(py),
//             })
//         })
//     }

//     fn execute_in_process<F, R>(&self, f: F) -> R
//     where
//         F: FnOnce() -> R + Send + 'static,
//         R: Send + 'static,
//     {
//         let (tx, rx) = mpsc::channel();

//         // Create a Python function that will run in a separate process
//         Python::with_gil(|py| {
//             let process_pool = self.process_pool.lock().unwrap();
//             let pool = process_pool[0].as_ref(py);

//             // Convert the Rust function to a Python callable
//             let py_func = PyDict::new(py);
//             py_func.set_item("func", move || {
//                 let result = f();
//                 tx.send(result).unwrap();
//             })?;

//             // Execute the function in a process
//             pool.call_method1("apply", (py_func.get_item("func").unwrap(),))?;

//             Ok(())
//         }).unwrap();

//         // Wait for and return the result
//         rx.recv().unwrap()
//     }
// }

// impl Executor for PythonExecutor {
//     fn execute<F, R>(&self, f: F) -> R
//     where
//         F: FnOnce() -> R + Send + 'static,
//         R: Send + 'static,
//     {
//         self.execute_in_process(f)
//     }

//     fn execute_batch<F, R>(&self, f: Vec<F>) -> Vec<R>
//     where
//         F: FnOnce() -> R + Send + 'static,
//         R: Send + 'static,
//     {
//         let (tx, rx) = mpsc::channel();
//         let mut handles = Vec::new();

//         // Create a Python function that will run in separate processes
//         Python::with_gil(|py| {
//             let process_pool = self.process_pool.lock().unwrap();
//             let pool = process_pool[0].as_ref(py);

//             for func in f {
//                 let tx = tx.clone();
//                 let py_func = PyDict::new(py);
//                 py_func.set_item("func", move || {
//                     let result = func();
//                     tx.send(result).unwrap();
//                 })?;

//                 // Execute each function in a process
//                 pool.call_method1("apply_async", (py_func.get_item("func").unwrap(),))?;
//             }

//             Ok(())
//         }).unwrap();

//         // Collect all results
//         (0..f.len()).map(|_| rx.recv().unwrap()).collect()
//     }

//     fn submit<F>(&self, f: F)
//     where
//         F: FnOnce() + Send + 'static,
//     {
//         let _ = self.execute_in_process(f);
//     }

//     fn submit_batch<F>(&self, f: Vec<F>)
//     where
//         F: FnOnce() + Send + 'static,
//     {
//         let _ = self.execute_batch(f);
//     }
// }

// impl Drop for PythonExecutor {
//     fn drop(&mut self) {
//         Python::with_gil(|py| {
//             let process_pool = self.process_pool.lock().unwrap();
//             let pool = process_pool[0].as_ref(py);

//             // Close and join the process pool
//             pool.call_method0("close")?;
//             pool.call_method0("join")?;

//             Ok(())
//         }).unwrap();
//     }
// }

// // Python bindings
// #[pymodule]
// fn executors(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
//     #[pyclass]
//     struct PyExecutor {
//         inner: PythonExecutor,
//     }

//     #[pymethods]
//     impl PyExecutor {
//         #[new]
//         fn new(num_processes: usize) -> PyResult<Self> {
//             Ok(Self {
//                 inner: PythonExecutor::new(num_processes)?,
//             })
//         }
//     }

//     m.add_class::<PyExecutor>()?;
//     Ok(())
// }
