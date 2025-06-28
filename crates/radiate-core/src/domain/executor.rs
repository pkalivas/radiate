use crate::thread_pool::{ThreadPool, WaitGroup};
#[cfg(feature = "rayon")]
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::{Arc, OnceLock};

struct FixedThreadPool {
    inner: Arc<ThreadPool>,
}

impl AsRef<ThreadPool> for FixedThreadPool {
    fn as_ref(&self) -> &ThreadPool {
        &self.inner
    }
}

impl FixedThreadPool {
    /// Returns the global instance of the registry.
    pub(self) fn instance(num_workers: usize) -> &'static FixedThreadPool {
        static INSTANCE: OnceLock<FixedThreadPool> = OnceLock::new();

        INSTANCE.get_or_init(|| FixedThreadPool {
            inner: Arc::new(ThreadPool::new(num_workers)),
        })
    }
}

fn get_thread_pool(num_workers: usize) -> &'static FixedThreadPool {
    &FixedThreadPool::instance(num_workers)
}

#[derive(Clone, Debug)]
pub enum Executor {
    Serial,
    #[cfg(feature = "rayon")]
    WorkerPool,
    #[cfg(not(feature = "rayon"))]
    WorkerPool(usize),
}

impl Default for Executor {
    fn default() -> Self {
        Executor::Serial
    }
}

impl Executor {
    pub fn num_workers(&self) -> usize {
        match self {
            Executor::Serial => 1,
            #[cfg(not(feature = "rayon"))]
            Executor::WorkerPool(num_workers) => *num_workers,
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
            Executor::Serial => f(),
            #[cfg(not(feature = "rayon"))]
            Executor::WorkerPool(num_workers) => get_thread_pool(*num_workers)
                .as_ref()
                .submit_with_result(f)
                .result(),
            #[cfg(feature = "rayon")]
            Executor::WorkerPool => {
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
            #[cfg(not(feature = "rayon"))]
            Executor::WorkerPool(num_workers) => {
                let pool = get_thread_pool(*num_workers).as_ref();
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
            Executor::WorkerPool => f.into_par_iter().map(|func| func()).collect(),
        }
    }

    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match self {
            Executor::Serial => f(),
            #[cfg(not(feature = "rayon"))]
            Executor::WorkerPool(num_workers) => {
                let pool = get_thread_pool(*num_workers).as_ref();
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
            #[cfg(not(feature = "rayon"))]
            Executor::WorkerPool(num_workers) => {
                let pool = get_thread_pool(*num_workers).as_ref();
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
