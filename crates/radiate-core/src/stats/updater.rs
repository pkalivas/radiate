use crate::{Metric, MetricSet, WaitGroup};
use std::{
    sync::{Arc, Mutex, mpsc},
    thread::JoinHandle,
};

enum BatchCommand {
    Update(Vec<Metric>),
    Complete,
}

pub struct BatchMetricUpdater {
    set: Arc<Mutex<MetricSet>>,
    thread: Option<JoinHandle<()>>,
    guard: Arc<WaitGroup>,
    sender: mpsc::Sender<BatchCommand>,
}

impl BatchMetricUpdater {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<BatchCommand>();

        let guard = Arc::new(WaitGroup::new());
        let receiver = Arc::new(Mutex::new(receiver));
        let set = Arc::new(Mutex::new(MetricSet::new()));
        let thread_set = Arc::clone(&set);
        let thread_guard = Arc::clone(&guard);
        let thread = std::thread::spawn(move || {
            loop {
                let updates = receiver.lock().unwrap().recv().unwrap();
                let waiter = thread_guard.guard();

                match updates {
                    BatchCommand::Update(metrics) => {
                        let mut set = thread_set.lock().unwrap();
                        for metric in metrics {
                            set.add_or_update(metric);
                        }
                    }
                    BatchCommand::Complete => return,
                };
                drop(waiter);
            }
        });

        BatchMetricUpdater {
            set,
            thread: Some(thread),
            guard,
            sender,
        }
    }

    #[inline]
    pub fn update(&self, update: Vec<Metric>) {
        self.sender.send(BatchCommand::Update(update)).unwrap();
    }

    #[inline]
    pub fn flush_into(&self, target: &mut MetricSet) {
        self.guard.wait();
        let mut set = self.set.lock().unwrap();
        set.flush_all_into(target);
        set.clear();
    }
}

impl Drop for BatchMetricUpdater {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = self.sender.send(BatchCommand::Complete);
            let _ = thread.join();
        }
    }
}
