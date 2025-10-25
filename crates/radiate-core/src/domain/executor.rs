use crate::thread_pool::WaitGroup;
use crate::thread_pool::get_thread_pool;
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

    pub fn execute_batch<F, R>(&self, f: Vec<F>) -> Vec<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        match self {
            Executor::Serial => f.into_iter().map(|func| func()).collect(),
            Executor::FixedSizedWorkerPool(num_workers) => {
                let pool = get_thread_pool(*num_workers);
                let mut results = Vec::with_capacity(f.len());

                for job in f {
                    results.push(pool.submit_with_result(|| job()));
                }

                results.into_iter().map(|r| r.result()).collect()
            }
            #[cfg(feature = "rayon")]
            Executor::WorkerPool => f.into_par_iter().map(|func| func()).collect(),
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

    pub fn submit_blocking<F>(&self, f: Vec<F>)
    where
        F: FnOnce() + Send + 'static,
    {
        match self {
            Executor::Serial => {
                for func in f {
                    func();
                }
            }
            Executor::FixedSizedWorkerPool(num_workers) => {
                let pool = get_thread_pool(*num_workers);
                let wg = WaitGroup::new();
                for job in f {
                    let wg_clone = wg.guard();
                    pool.submit(move || {
                        job();
                        drop(wg_clone);
                    });
                }

                wg.wait();
            }
            #[cfg(feature = "rayon")]
            Executor::WorkerPool => {
                let wg = WaitGroup::new();
                let with_guards = f
                    .into_iter()
                    .map(|job| {
                        let _wg_clone = wg.guard();
                        move || {
                            job();
                            drop(_wg_clone);
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

        let batch = vec![|| 1 * 2, || 2 * 2, || 3 * 2];
        let results = executor.execute_batch(batch);
        assert_eq!(results, vec![2, 4, 6]);
    }

    #[test]
    fn test_executor_fixed_sized_worker_pool() {
        let executor = Executor::FixedSizedWorkerPool(4);
        let result = executor.execute(|| 42);

        let batch = vec![|| 1 * 2, || 2 * 2, || 3 * 2];
        let results = executor.execute_batch(batch);

        assert_eq!(executor.num_workers(), 4);
        assert_eq!(result, 42);
        assert_eq!(results, vec![2, 4, 6]);
    }
}
