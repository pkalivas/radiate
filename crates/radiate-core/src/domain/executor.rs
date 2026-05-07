use crate::sync::WaitGroup;
use crate::sync::get_thread_pool;
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Clone, Debug, Default)]
pub enum Executor {
    #[default]
    Serial,
    #[cfg(feature = "rayon")]
    WorkerPool,
    FixedSizedWorkerPool(usize),
}

impl Executor {
    pub fn is_parallel(&self) -> bool {
        match self {
            Executor::Serial => false,
            #[cfg(feature = "rayon")]
            Executor::WorkerPool => true,
            Executor::FixedSizedWorkerPool(_) => true,
        }
    }

    pub fn num_workers(&self) -> usize {
        match self {
            Executor::Serial => 1,
            #[cfg(feature = "rayon")]
            Executor::WorkerPool => rayon::current_num_threads(),
            Executor::FixedSizedWorkerPool(num_workers) => *num_workers,
        }
    }

    pub fn execute<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        match self {
            Executor::Serial => f(),
            Executor::FixedSizedWorkerPool(num_workers) => {
                get_thread_pool(*num_workers).submit_with_result(f).result()
            }
            #[cfg(feature = "rayon")]
            Executor::WorkerPool => {
                use std::sync::{Arc, Mutex};

                let result = Arc::new(Mutex::new(None));
                let result_clone = Arc::clone(&result);
                let wg = WaitGroup::new();
                let _wg_clone = wg.guard();
                rayon::spawn_fifo(move || {
                    let res = f();
                    let mut guard = result_clone.lock().unwrap();
                    *guard = Some(res);
                    drop(_wg_clone);
                });

                wg.wait();

                (*result.lock().unwrap()).take().unwrap()
            }
        }
    }

    pub fn execute_batch<I, F, R>(&self, jobs: I) -> Vec<R>
    where
        I: IntoIterator<Item = F>,
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        match self {
            Executor::Serial => jobs.into_iter().map(|func| func()).collect(),
            Executor::FixedSizedWorkerPool(num_workers) => {
                let pool = get_thread_pool(*num_workers);
                let iter = jobs.into_iter();
                let mut results = Vec::with_capacity(iter.size_hint().0);

                for job in iter {
                    results.push(pool.submit_with_result(job));
                }

                results.into_iter().map(|r| r.result()).collect()
            }
            #[cfg(feature = "rayon")]
            Executor::WorkerPool => jobs
                .into_iter()
                .collect::<Vec<_>>()
                .into_par_iter()
                .map(|func| func())
                .collect(),
        }
    }

    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match self {
            Executor::Serial => f(),
            Executor::FixedSizedWorkerPool(num_workers) => {
                let pool = get_thread_pool(*num_workers);
                pool.submit(f)
            }
            #[cfg(feature = "rayon")]
            Executor::WorkerPool => {
                rayon::spawn_fifo(move || {
                    f();
                });
            }
        }
    }

    pub fn submit_blocking<I, F>(&self, jobs: I)
    where
        I: IntoIterator<Item = F>,
        F: FnOnce() + Send + 'static,
    {
        match self {
            Executor::Serial => {
                for func in jobs {
                    func();
                }
            }
            Executor::FixedSizedWorkerPool(num_workers) => {
                let pool = get_thread_pool(*num_workers);
                let wg = WaitGroup::new();
                for job in jobs {
                    let guard = wg.guard();
                    pool.submit(move || {
                        job();
                        drop(guard);
                    });
                }

                wg.wait();
            }
            #[cfg(feature = "rayon")]
            Executor::WorkerPool => {
                let wg = WaitGroup::new();
                let with_guards = jobs
                    .into_iter()
                    .map(|job| {
                        let guard = wg.guard();
                        move || {
                            job();
                            drop(guard);
                        }
                    })
                    .collect::<Vec<_>>();

                with_guards.into_par_iter().for_each(|func| {
                    func();
                });

                wg.wait();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Executor;

    #[test]
    fn test_executor_serial() {
        let executor = Executor::Serial;
        let result = executor.execute(|| 42);
        assert_eq!(result, 42);

        let batch: Vec<Box<dyn FnOnce() -> i32 + Send>> =
            vec![Box::new(|| 1 * 2), Box::new(|| 2 * 2), Box::new(|| 3 * 2)];
        let results = executor.execute_batch(batch);
        assert_eq!(results, vec![2, 4, 6]);
    }

    #[test]
    fn test_executor_fixed_sized_worker_pool() {
        let executor = Executor::FixedSizedWorkerPool(4);
        let result = executor.execute(|| 42);

        let batch: Vec<Box<dyn FnOnce() -> i32 + Send>> =
            vec![Box::new(|| 1 * 2), Box::new(|| 2 * 2), Box::new(|| 3 * 2)];
        let results = executor.execute_batch(batch);

        assert_eq!(executor.num_workers(), 4);
        assert_eq!(result, 42);
        assert_eq!(results, vec![2, 4, 6]);
    }
}
