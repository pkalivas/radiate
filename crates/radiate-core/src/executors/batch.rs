// use crate::thread_pool::{ThreadPool, WaitGroup};
// use std::sync::Arc;

// use super::Executor;

// #[derive(Clone)]
// pub struct BatchExecutor {
//     thread_pool: Arc<ThreadPool>,
// }

// impl BatchExecutor {
//     pub fn new(num_batches: usize) -> Self {
//         Self {
//             thread_pool: Arc::new(ThreadPool::new(num_batches)),
//         }
//     }
// }

// impl Executor for BatchExecutor {
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
//         let wg = WaitGroup::new();
//         let mut results = Vec::with_capacity(f.len());
//         for job in f {
//             let wg_clone = wg.guard();
//             let result = self.thread_pool.submit_with_result(move || {
//                 let res = job();
//                 drop(wg_clone);
//                 res
//             });
//             results.push(result);
//         }
//         wg.wait();
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
