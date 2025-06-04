use crate::thread_pool::{ThreadPool, WaitGroup};

pub enum Executor {
    Serial,
    WorkerPool(ThreadPool),
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
        }
    }

    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match self {
            Executor::Serial => f(),
            Executor::WorkerPool(pool) => pool.submit(f),
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
        }
    }
}
