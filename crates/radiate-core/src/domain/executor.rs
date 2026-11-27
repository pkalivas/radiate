use crate::sync::{ThreadPool, WaitGroup};
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub enum Executor {
    #[default]
    Inline,
    ThreadPool(Arc<ThreadPool>),
    #[cfg(feature = "rayon")]
    WorkerPool,
}

impl Executor {
    pub fn new_thread_pool(num_workers: usize) -> Self {
        Executor::ThreadPool(Arc::new(ThreadPool::new(num_workers)))
    }

    pub fn is_parallel(&self) -> bool {
        match self {
            Executor::Inline => false,
            Executor::ThreadPool(_) => true,
            #[cfg(feature = "rayon")]
            Executor::WorkerPool => true,
        }
    }

    pub fn num_workers(&self) -> usize {
        match self {
            Executor::Inline => 1,
            Executor::ThreadPool(pool) => pool.num_workers(),
            #[cfg(feature = "rayon")]
            Executor::WorkerPool => rayon::current_num_threads(),
        }
    }

    pub fn execute<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        match self {
            Executor::Inline => f(),
            Executor::ThreadPool(pool) => pool.submit_with_result(f).result(),
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
            Executor::Inline => jobs.into_iter().map(|func| func()).collect(),
            Executor::ThreadPool(pool) => {
                let (tx, rx) = crossbeam::channel::unbounded();

                let mut count = 0;
                for (i, job) in jobs.into_iter().enumerate() {
                    let tx = tx.clone();
                    pool.submit(move || {
                        tx.send((i, job())).unwrap();
                    });
                    count += 1;
                }

                // so `rx` disconnects once the last job's sender is gone
                drop(tx);

                let mut slots = (0..count).map(|_| None).collect::<Vec<_>>();
                for (i, result) in rx.iter() {
                    slots[i] = Some(result);
                }

                slots.into_iter().map(|r| r.unwrap()).collect()
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
            Executor::Inline => f(),
            Executor::ThreadPool(pool) => pool.submit(f),
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
            Executor::Inline => {
                for func in jobs {
                    func();
                }
            }
            Executor::ThreadPool(pool) => {
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
        let executor = Executor::Inline;
        let result = executor.execute(|| 42);
        assert_eq!(result, 42);

        let batch: Vec<Box<dyn FnOnce() -> i32 + Send>> =
            vec![Box::new(|| 2), Box::new(|| 2 * 2), Box::new(|| 3 * 2)];
        let results = executor.execute_batch(batch);
        assert_eq!(results, vec![2, 4, 6]);
    }

    #[test]
    fn test_executor_fixed_sized_worker_pool() {
        let executor = Executor::new_thread_pool(4);
        let result = executor.execute(|| 42);

        let batch: Vec<Box<dyn FnOnce() -> i32 + Send>> =
            vec![Box::new(|| 2), Box::new(|| 2 * 2), Box::new(|| 3 * 2)];
        let results = executor.execute_batch(batch);

        assert_eq!(executor.num_workers(), 4);
        assert_eq!(result, 42);
        assert_eq!(results, vec![2, 4, 6]);
    }
}
