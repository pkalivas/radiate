use std::{fmt::Debug, sync::Arc};

use super::Executor;
use crate::thread_pool::{ThreadPool, WaitGroup};

#[derive(Clone)]
pub struct WorkerPoolExecutor {
    thread_pool: Arc<ThreadPool>,
}

impl WorkerPoolExecutor {
    pub fn new(num_threads: usize) -> Self {
        Self {
            thread_pool: Arc::new(ThreadPool::new(num_threads)),
        }
    }

    pub fn with_thread_pool(thread_pool: Arc<ThreadPool>) -> Self {
        Self { thread_pool }
    }

    pub fn num_workers(&self) -> usize {
        self.thread_pool.num_workers()
    }
}

// impl Executor for WorkerPoolExecutor {
//     fn execute<F, R>(&self, f: F) -> R
//     where
//         F: FnOnce() -> R + Send + 'static,
//         R: Send + 'static,
//     {
//         self.thread_pool.submit_with_result(f).result()
//     }

//     fn execute_batch<F, R>(&self, f: Vec<F>) -> Vec<R>
//     where
//         F: FnOnce() -> R + Send + 'static,
//         R: Send + 'static,
//     {
//         let mut results = Vec::with_capacity(f.len());
//         for job in f {
//             let result = self.thread_pool.submit_with_result(move || job());
//             results.push(result);
//         }

//         results.into_iter().map(|r| r.result()).collect()
//     }

//     fn submit<F>(&self, f: F)
//     where
//         F: FnOnce() + Send + 'static,
//     {
//         self.thread_pool.submit(f);
//     }

//     fn submit_batch<F>(&self, f: Vec<F>)
//     where
//         F: FnOnce() + Send + 'static,
//     {
//         let wg = WaitGroup::new();
//         for job in f {
//             let wg_clone = wg.guard();
//             self.thread_pool.submit(move || {
//                 job();
//                 drop(wg_clone);
//             });
//         }
//         wg.wait();
//     }
// }

impl Default for WorkerPoolExecutor {
    fn default() -> Self {
        Self::new(1)
    }
}

impl Debug for WorkerPoolExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkerPoolExecutor")
            .field("num_threads", &self.thread_pool.num_workers())
            .finish()
    }
}
