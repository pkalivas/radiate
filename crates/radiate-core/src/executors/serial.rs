use super::Executor;

#[derive(Debug, Clone, Default)]
pub struct SerialExecutor;

impl SerialExecutor {
    pub fn new() -> Self {
        SerialExecutor
    }
}

impl Executor for SerialExecutor {
    fn execute<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        f()
    }

    fn execute_batch<F, R>(&self, f: Vec<F>) -> Vec<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        f.into_iter().map(|func| func()).collect()
    }

    fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        f();
    }

    fn submit_batch<F>(&self, f: Vec<F>)
    where
        F: FnOnce() + Send + 'static,
    {
        for func in f {
            func();
        }
    }
}
