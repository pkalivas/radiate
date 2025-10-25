use std::{
    fmt::Debug,
    sync::{
        Arc, Condvar, Mutex, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
};
use std::{sync::mpsc, thread};

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

pub(crate) fn get_thread_pool(num_workers: usize) -> Arc<ThreadPool> {
    Arc::clone(&FixedThreadPool::instance(num_workers).inner)
}

/// [WorkResult] is a simple wrapper around a `Receiver` that allows the user to get
/// the result of a job that was executed in the thread pool. It kinda acts like
/// a `Future` in a synchronous way.
pub struct WorkResult<T> {
    receiver: mpsc::Receiver<T>,
}

impl<T> WorkResult<T> {
    /// Get the result of the job.
    /// **Note**: This method will block until the result is available.
    pub fn result(&self) -> T {
        self.receiver.recv().unwrap()
    }
}

pub struct ThreadPool {
    sender: mpsc::Sender<Message>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    /// Basic thread pool implementation.
    ///
    /// Create a new ThreadPool with the given size.
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        ThreadPool {
            sender,
            workers: (0..size)
                .map(|id| Worker::new(id, Arc::clone(&receiver)))
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
        let (tx, rx) = mpsc::sync_channel(1);
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
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        Worker {
            id,
            thread: Some(thread::spawn(move || {
                loop {
                    let message = receiver.lock().unwrap().recv().unwrap();

                    match message {
                        Message::Work(job) => job(),
                        Message::Terminate => break,
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

#[derive(Clone)]
pub struct WaitGroup {
    inner: Arc<Inner>,
    total_count: Arc<AtomicUsize>,
}

struct Inner {
    counter: AtomicUsize,
    lock: Mutex<()>,
    cvar: Condvar,
}

pub struct WaitGuard {
    wg: WaitGroup,
}

impl Drop for WaitGuard {
    fn drop(&mut self) {
        if self.wg.inner.counter.fetch_sub(1, Ordering::AcqRel) == 1 {
            let _guard = self.wg.inner.lock.lock().unwrap();
            self.wg.inner.cvar.notify_all();
        }
    }
}

impl WaitGroup {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                counter: AtomicUsize::new(0),
                lock: Mutex::new(()),
                cvar: Condvar::new(),
            }),
            total_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get_count(&self) -> usize {
        self.total_count.load(Ordering::Acquire)
    }

    pub fn guard(&self) -> WaitGuard {
        self.inner.counter.fetch_add(1, Ordering::AcqRel);
        self.total_count.fetch_add(1, Ordering::AcqRel);
        WaitGuard { wg: self.clone() }
    }

    /// Waits until the counter reaches zero.
    pub fn wait(&self) -> usize {
        if self.inner.counter.load(Ordering::Acquire) == 0 {
            return 0;
        }

        let lock = self.inner.lock.lock().unwrap();
        let _unused = self
            .inner
            .cvar
            .wait_while(lock, |_| self.inner.counter.load(Ordering::Acquire) != 0);

        self.get_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_thread_pool_creation() {
        let pool = ThreadPool::new(4);
        assert!(pool.is_alive());
    }

    #[test]
    fn test_basic_job_execution() {
        let pool = ThreadPool::new(4);
        let counter = Arc::new(Mutex::new(0));

        for _ in 0..8 {
            let counter = Arc::clone(&counter);
            pool.submit(move || {
                let mut num = counter.lock().unwrap();
                *num += 1;
            });
        }

        // Give threads some time to finish processing
        thread::sleep(Duration::from_secs(1));
        assert_eq!(*counter.lock().unwrap(), 8);
    }

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(4);

        for i in 0..8 {
            pool.submit(move || {
                let start_time = std::time::SystemTime::now();
                println!("Job {} started.", i);
                thread::sleep(Duration::from_secs(1));
                println!("Job {} finished in {:?}.", i, start_time.elapsed().unwrap());
            });
        }
    }

    #[test]
    fn test_job_order() {
        let pool = ThreadPool::new(2);
        let results = Arc::new(Mutex::new(vec![]));

        for i in 0..5 {
            let results = Arc::clone(&results);
            pool.submit(move || {
                results.lock().unwrap().push(i);
            });
        }

        // Give threads some time to finish processing
        thread::sleep(Duration::from_secs(1));
        let mut results = results.lock().unwrap();
        results.sort(); // Order may not be guaranteed
        assert_eq!(*results, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_thread_pool_process() {
        let pool = ThreadPool::new(4);

        let results = pool.submit_with_result(|| {
            let start_time = std::time::SystemTime::now();
            println!("Job started.");
            thread::sleep(Duration::from_secs(2));
            println!("Job finished in {:?}.", start_time.elapsed().unwrap());
            42
        });

        let result = results.result();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_max_concurrent_jobs() {
        let pool = ThreadPool::new(4);
        let (tx, rx) = mpsc::channel();
        let num_jobs = 20;
        let start_time = Instant::now();

        // Submit 20 jobs
        for i in 0..num_jobs {
            let tx = tx.clone();
            pool.submit(move || {
                thread::sleep(Duration::from_millis(100));
                tx.send(i).unwrap();
            });
        }

        // Wait for all jobs to finish
        let mut results = vec![];
        for _ in 0..num_jobs {
            results.push(rx.recv().unwrap());
        }

        let elapsed = start_time.elapsed();
        assert!(elapsed < Duration::from_secs(3));
        assert_eq!(results.len(), num_jobs);
        assert!(results.iter().all(|&x| x < num_jobs));
    }

    #[test]
    fn tests_thread_pool_submit_with_result_returns_correct_order() {
        let pool = ThreadPool::new(5);
        let num_jobs = 10;
        let mut work_results = vec![];

        for i in 0..num_jobs {
            let work_result = pool.submit_with_result(move || {
                thread::sleep(Duration::from_millis(50 * (num_jobs - i) as u64));
                i * i
            });
            work_results.push(work_result);
        }

        for (i, work_result) in work_results.into_iter().enumerate() {
            let result = work_result.result();
            assert_eq!(result, i * i);
        }
    }

    #[test]
    fn test_wait_group() {
        let pool = ThreadPool::new(4);
        let wg = WaitGroup::new();
        let num_tasks = 10;
        let total = Arc::new(Mutex::new(0));

        for _ in 0..num_tasks {
            let guard = wg.guard();
            let total = Arc::clone(&total);
            pool.submit(move || {
                thread::sleep(Duration::from_millis(100));
                let mut num = total.lock().unwrap();
                *num += 1;
                drop(guard);
            });
        }

        // Not all tasks should be done yet - so the total should be less than num_tasks
        {
            let total = total.lock().unwrap();
            assert_ne!(*total, num_tasks);
        }

        let total_tasks_waited_for = wg.wait();

        // Now all tasks should be done - so the total should equal num_tasks
        let total = total.lock().unwrap();
        assert_eq!(*total, num_tasks);
        assert_eq!(total_tasks_waited_for, num_tasks);
    }

    #[test]
    fn test_wait_group_zero_tasks() {
        let wg = WaitGroup::new();
        let total_tasks_waited_for = wg.wait();
        assert_eq!(total_tasks_waited_for, 0);
    }
}
