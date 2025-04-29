use std::{
    sync::atomic::{AtomicUsize, Ordering},
    sync::{Arc, Condvar, Mutex},
};
use std::{sync::mpsc, thread};

/// `WorkResult` is a simple wrapper around a `Receiver` that allows the user to get
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
                .map(|_| Worker::new(Arc::clone(&receiver)))
                .collect(),
        }
    }

    pub fn group_submit(&self, wg: &WaitGroup, f: impl FnOnce() + Send + 'static) {
        let guard = wg.guard();

        self.submit(move || {
            f();
            drop(guard);
        });
    }

    /// Execute a job in the thread pool. This is a 'fire and forget' method.
    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }

    /// Execute a job in the thread pool and return a `WorkResult` that can be used to get the result of the job.
    pub fn submit_with_result<F, T>(&self, f: F) -> WorkResult<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::sync_channel(1);
        let job = Box::new(move || tx.send(f()).unwrap());

        self.sender.send(Message::NewJob(job)).unwrap();
        WorkResult { receiver: rx }
    }

    pub fn num_workers(&self) -> usize {
        self.workers.len()
    }

    pub fn is_alive(&self) -> bool {
        self.workers.iter().any(|worker| worker.is_alive())
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
    NewJob(Job),
    Terminate,
}

/// Worker struct that listens for incoming `Message`s and executes the `Job`s or terminates.
struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    /// Create a new Worker.
    ///
    /// The Worker will listen for incoming jobs on the given receiver.
    /// When a job is received, it will be executed in a new thread and the
    /// mutex will release allowing another job to be received from a different worker.
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        Worker {
            thread: Some(thread::spawn(move || {
                loop {
                    let message = receiver.lock().unwrap().recv().unwrap();

                    match message {
                        Message::NewJob(job) => job(),
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

    /// Adds one to the counter and returns a scoped guard that will decrement when dropped.
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
    use std::time::{Duration, Instant};

    use super::*;

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
}
