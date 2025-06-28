use crate::thread_pool::{ThreadPool, WaitGroup};
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, ParallelIterator};

#[derive(Default)]
pub enum Executor {
    #[default]
    Serial,
    WorkerPool(ThreadPool),
    #[cfg(feature = "rayon")]
    Rayon,
}

impl Executor {
    pub fn serial() -> Self {
        Executor::Serial
    }

    #[cfg(not(feature = "rayon"))]
    pub fn worker_pool(num_workers: usize) -> Self {
        if num_workers == 1 {
            return Executor::Serial;
        }

        let pool = ThreadPool::new(num_workers);
        Executor::WorkerPool(pool)
    }

    #[cfg(feature = "rayon")]
    pub fn worker_pool(num_workers: usize) -> Self {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_workers)
            .build()
            .map(|_| Executor::Rayon)
            .unwrap_or_else(|_| {
                if num_workers == 1 {
                    Executor::Serial
                } else {
                    let pool = ThreadPool::new(num_workers);
                    Executor::WorkerPool(pool)
                }
            })
    }

    pub fn num_workers(&self) -> usize {
        match self {
            Executor::Serial => 1,
            Executor::WorkerPool(pool) => pool.num_workers(),
            #[cfg(feature = "rayon")]
            Executor::Rayon => rayon::current_num_threads(),
        }
    }

    pub fn execute<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        match self {
            Executor::Serial => f(),
            Executor::WorkerPool(pool) => pool.submit_with_result(f).result(),
            #[cfg(feature = "rayon")]
            Executor::Rayon => {
                use std::sync::{Arc, Mutex};

                let result: Arc<Mutex<Option<R>>> = Arc::new(Mutex::new(None));
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
            Executor::WorkerPool(pool) => {
                let wg = WaitGroup::new();
                let mut results = Vec::with_capacity(f.len());

                for job in f {
                    let wg_clone = wg.guard();
                    let result = pool.submit_with_result(move || {
                        let res = job();
                        drop(wg_clone);
                        res
                    });

                    results.push(result);
                }

                wg.wait();

                results.into_iter().map(|r| r.result()).collect()
            }
            #[cfg(feature = "rayon")]
            Executor::Rayon => f.into_par_iter().map(|func| func()).collect(),
        }
    }

    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match self {
            Executor::Serial => f(),
            Executor::WorkerPool(pool) => pool.submit(f),
            #[cfg(feature = "rayon")]
            Executor::Rayon => {
                rayon::spawn_fifo(move || {
                    f();
                });
            }
        }
    }

    pub fn submit_batch<F>(&self, f: Vec<F>)
    where
        F: FnOnce() + Send + 'static,
    {
        match self {
            Executor::Serial => {
                for func in f {
                    func();
                }
            }
            Executor::WorkerPool(pool) => {
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
            Executor::Rayon => {
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
