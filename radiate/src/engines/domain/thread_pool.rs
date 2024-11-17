use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
};

pub struct WorkResult<T> {
    reseiver: mpsc::Receiver<T>,
}

impl<T> WorkResult<T> {
    pub fn result(&self) -> T {
        self.reseiver.recv().unwrap()
    }
}

pub struct ThreadPool {
    sender: mpsc::Sender<Message>,
    workers: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        Self {
            sender,
            workers: (0..size)
                .map(|_| Worker::new(Arc::clone(&receiver)))
                .collect(),
        }
    }

    pub fn submit<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }

    pub fn process<F, T>(&self, f: F) -> WorkResult<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let job = Box::new(move || {
            let result = f();
            tx.send(result).unwrap();
        });

        self.sender.send(Message::NewJob(job)).unwrap();
        WorkResult { reseiver: rx }
    }

    pub fn is_alive(&self) -> bool {
        self.workers.iter().any(|worker| worker.is_alive())
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        for worker in &mut self.workers {
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }

        assert!(!self.is_alive());
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Self {
        Self {
            thread: Some(thread::spawn(move || loop {
                let job = receiver.lock().unwrap().recv().unwrap();

                match job {
                    Message::NewJob(job) => job(),
                    Message::Terminate => break,
                }
            })),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.thread.is_some()
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
        thread::sleep(std::time::Duration::from_secs(1));
        assert_eq!(*counter.lock().unwrap(), 8);
    }

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(4);

        for i in 0..8 {
            pool.submit(move || {
                let start_time = std::time::SystemTime::now();
                println!("Job {} started.", i);
                thread::sleep(std::time::Duration::from_secs(1));
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
        thread::sleep(std::time::Duration::from_secs(1));
        let mut results = results.lock().unwrap();
        results.sort(); // Order may not be guaranteed
        assert_eq!(*results, vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_thread_pool_process() {
        let pool = ThreadPool::new(4);

        let results = pool.process(|| {
            let start_time = std::time::SystemTime::now();
            println!("Job started.");
            thread::sleep(std::time::Duration::from_secs(2));
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
