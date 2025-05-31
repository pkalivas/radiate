use crate::thread_pool::{ThreadPool, WaitGroup};

pub trait Processor: Send + Sync {
    fn num_workers(&self) -> usize;
}

pub enum Executor {
    Serial,
    WorkerPool(ThreadPool),
    Other(Box<dyn Processor>),
}

impl Executor {
    pub fn serial() -> Self {
        Executor::Serial
    }

    pub fn worker_pool(num_workers: usize) -> Self {
        let pool = ThreadPool::new(num_workers);
        Executor::WorkerPool(pool)
    }

    pub fn num_workers(&self) -> usize {
        match self {
            Executor::Serial => 1,
            Executor::WorkerPool(pool) => pool.num_workers(),
            Executor::Other(processor) => processor.num_workers(),
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
            Executor::Other(_) => f(),
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
                let mut results = Vec::with_capacity(f.len());
                for job in f {
                    let result = pool.submit_with_result(move || job());
                    results.push(result);
                }

                results.into_iter().map(|r| r.result()).collect()
            }
            Executor::Other(_) => f.into_iter().map(|func| func()).collect(),
        }
    }

    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match self {
            Executor::Serial => f(),
            Executor::WorkerPool(pool) => pool.submit(f),
            Executor::Other(_) => f(),
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
            Executor::Other(_) => {
                for func in f {
                    func();
                }
            }
        }
    }
}
