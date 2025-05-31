mod batch;
mod serial;
mod workers;

use std::ops::Deref;

pub use batch::BatchExecutor;
pub use serial::SerialExecutor;
pub use workers::WorkerPoolExecutor;

pub struct ExecutorHandle<E: Executor> {
    executor: E,
}

impl<E: Executor> ExecutorHandle<E> {
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

impl<E: Executor> Deref for ExecutorHandle<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.executor
    }
}

pub trait Executor {
    fn execute<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static;

    fn execute_batch<F, R>(&self, f: Vec<F>) -> Vec<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static;

    fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static;

    fn submit_batch<F>(&self, f: Vec<F>)
    where
        F: FnOnce() + Send + 'static;
}
