//! A small local executor for the playground (`Inline` to start; a minimal `std::thread` +
//! `crossbeam` fixed pool as a second mode). Deliberately not `radiate_core::Executor` —
//! `radiate-utils` sits below `radiate-core` in the dependency graph and can't depend on it.

use crossbeam::channel;
use std::thread;
use std::{
    fmt::Debug,
    sync::{Arc, OnceLock},
};

/// A fixed-size thread pool implementation. This thread pool will create a fixed number of worker threads
/// that will be reused for executing jobs. This is useful for limiting the number of concurrent threads
/// in the application.
///
/// The thread pool within the `FixedThreadPool` is created only once and will be reused for the lifetime of the program.
/// Meaning that the first time you request a thread pool with a specific number of workers, that number will be used.
/// Subsequent requests with different numbers will be ignored.
struct FixedThreadPool {
    inner: Arc<ThreadPool>,
}

impl FixedThreadPool {
    /// Returns the global instance of the threadpool.
    ///
    /// This thread pool is fixed in size and will be created only once. This means that
    /// the first time you call this method with a specific number of workers, that number will be used
    /// for the lifetime of the program. Subsequent calls with different numbers will be ignored.
    pub(self) fn instance(num_workers: usize) -> &'static FixedThreadPool {
        static INSTANCE: OnceLock<FixedThreadPool> = OnceLock::new();

        INSTANCE.get_or_init(|| FixedThreadPool {
            inner: Arc::new(ThreadPool::new(num_workers)),
        })
    }
}

pub fn get_thread_pool(num_workers: usize) -> Arc<ThreadPool> {
    Arc::clone(&FixedThreadPool::instance(num_workers).inner)
}

/// [WorkResult] is a simple wrapper around a `Receiver` that allows the user to get
/// the result of a job that was executed in the thread pool. It kinda acts like
/// a `Future` in a synchronous way.
pub struct WorkResult<T> {
    receiver: channel::Receiver<T>,
}

impl<T> WorkResult<T> {
    pub fn new(rx: channel::Receiver<T>) -> Self {
        WorkResult { receiver: rx }
    }
    /// Get the result of the job.
    /// **Note**: This method will block until the result is available.
    pub fn result(&self) -> T {
        self.receiver.recv().unwrap()
    }
}

pub struct ThreadPool {
    sender: channel::Sender<Message>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    /// Basic thread pool implementation.
    ///
    /// Create a new ThreadPool with the given size.
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = channel::unbounded();

        ThreadPool {
            sender,
            workers: (0..size)
                .map(|id| Worker::new(id, receiver.clone()))
                .collect(),
        }
    }

    pub fn num_workers(&self) -> usize {
        self.workers.len()
    }

    pub fn is_alive(&self) -> bool {
        self.workers.iter().any(|worker| worker.is_alive())
    }

    /// Execute a job in the thread pool. This method does not return anything
    /// and as such can be thought of as a 'fire-and-forget' job submission.
    ///
    /// # Example
    /// ```rust,ignore
    /// use radiate_core::domain::thread_pool::ThreadPool;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let pool = ThreadPool::new(4);
    /// let counter = Arc::new(Mutex::new(0));
    ///
    /// for _ in 0..8 {
    ///     let counter = Arc::clone(&counter);
    ///     pool.submit(move || {
    ///         let mut num = counter.lock().unwrap();
    ///         *num += 1;
    ///     });
    /// }
    ///
    /// // Drop the pool to join all threads
    /// drop(pool);
    ///
    /// assert_eq!(*counter.lock().unwrap(), 8);
    /// ```
    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::Work(job)).unwrap();
    }

    /// Execute a job in the thread pool and return a [WorkResult]
    /// that can be used to get the result of the job. This method
    /// is similar to a 'future' in that it allows the user to get
    /// the result of the job at a later time. It should be noted that the [WorkResult]
    /// will block when calling `result()` until the job is complete.
    ///
    /// # Example
    /// ```rust,ignore
    /// use radiate_core::domain::thread_pool::ThreadPool;
    ///
    /// let pool = ThreadPool::new(4);
    /// let work_result = pool.submit_with_result(|| 10 + 32);
    ///
    /// // Drop the pool to join all threads
    /// drop(pool);
    ///
    /// let result = work_result.result();
    /// assert_eq!(result, 42);
    /// ```
    pub fn submit_with_result<F, T>(&self, f: F) -> WorkResult<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = channel::bounded(1);
        let job = Box::new(move || tx.send(f()).unwrap());

        self.sender.send(Message::Work(job)).unwrap();

        WorkResult { receiver: rx }
    }
}

/// Drop implementation for ThreadPool. This will terminate all workers when the ThreadPool is dropped.
/// We need to make sure that all workers are terminated before the ThreadPool is dropped.
impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in self.workers.iter() {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in self.workers.iter_mut() {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }

        assert!(!self.is_alive());
    }
}

/// Job type that can be executed in the thread pool.
type Job = Box<dyn FnOnce() + Send + 'static>;

/// Message type that can be sent to the worker threads.
enum Message {
    Work(Job),
    Terminate,
}

/// Worker struct that listens for incoming `Message`s and executes the `Job`s or terminates.
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Create a new Worker.
    ///
    /// Runs jobs on a long-lived worker thread that pulls tasks from the queue.
    fn new(id: usize, receiver: channel::Receiver<Message>) -> Self {
        Worker {
            id,
            thread: Some(thread::spawn(move || {
                loop {
                    while let Ok(message) = receiver.recv() {
                        match message {
                            Message::Work(job) => job(),
                            Message::Terminate => return,
                        }
                    }
                }
            })),
        }
    }

    /// Simple check if the worker is alive. The thread is 'taken' when the worker is dropped.
    /// So if the thread is 'None' the worker is no longer alive.
    pub fn is_alive(&self) -> bool {
        self.thread.is_some()
    }
}

impl Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Worker")
            .field("id", &self.id)
            .field("is_alive", &self.is_alive())
            .finish()
    }
}

/// The pluggable dispatch strategy `Subscriber`/`Hub` submit work through. `Inline` runs a
/// job immediately on the calling thread (what everything before this step effectively did);
/// `FixedSizedWorkerPool` hands it off to the thread pool above, so a publisher can return
/// before the handler actually runs.
#[derive(Clone, Debug, Default)]
pub(super) enum Executor {
    #[default]
    Inline,
    FixedSizedWorkerPool(usize),
}

impl Executor {
    pub(super) fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        match self {
            Executor::Inline => f(),
            Executor::FixedSizedWorkerPool(num_workers) => {
                get_thread_pool(*num_workers).submit(f)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn inline_runs_on_the_calling_thread_synchronously() {
        let ran = Arc::new(AtomicUsize::new(0));
        let ran_clone = Arc::clone(&ran);

        Executor::Inline.submit(move || {
            ran_clone.fetch_add(1, Ordering::SeqCst);
        });

        // No hand-off at all — already done by the time `submit` returns.
        assert_eq!(ran.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn fixed_pool_runs_off_the_calling_thread() {
        let caller = thread::current().id();
        let (tx, rx) = channel::bounded(1);

        // `get_thread_pool` is a process-wide singleton keyed by whichever size asks for it
        // first (see `FixedThreadPool::instance`'s doc comment) — every `FixedSizedWorkerPool`
        // use across this crate's tests sticks to 4 for that reason, so the "N workers" in
        // any printed throughput numbers is always the size actually in effect.
        Executor::FixedSizedWorkerPool(4).submit(move || {
            tx.send(thread::current().id()).unwrap();
        });

        let worker = rx.recv().unwrap();
        assert_ne!(worker, caller);
    }

    #[test]
    fn thread_pool_runs_jobs_across_all_workers() {
        const JOBS: usize = 50;
        let pool = ThreadPool::new(4);
        let seen = Arc::new(Mutex::new(Vec::new()));

        let mut results = Vec::new();
        for i in 0..JOBS {
            let seen = Arc::clone(&seen);
            results.push(pool.submit_with_result(move || {
                seen.lock().unwrap().push(thread::current().id());
                i
            }));
        }

        let values: Vec<_> = results.iter().map(|r| r.result()).collect();
        assert_eq!(values.len(), JOBS);

        drop(pool);
        assert!(!seen.lock().unwrap().is_empty());
    }
}
