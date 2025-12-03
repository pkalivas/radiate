use crate::{Metric, MetricSet, WaitGroup};
use std::{
    sync::{Arc, Mutex, mpsc},
    thread::JoinHandle,
};

pub enum MetricSetUpdate {
    Many(Vec<Metric>),
    Single(Metric),
    Fn(Box<dyn FnOnce(&mut MetricSet) + Send>),
}

impl From<Vec<Metric>> for MetricSetUpdate {
    fn from(metrics: Vec<Metric>) -> Self {
        MetricSetUpdate::Many(metrics)
    }
}

impl From<Metric> for MetricSetUpdate {
    fn from(metric: Metric) -> Self {
        MetricSetUpdate::Single(metric)
    }
}

impl<F> From<F> for MetricSetUpdate
where
    F: FnOnce(&mut MetricSet) + Send + 'static,
{
    fn from(func: F) -> Self {
        MetricSetUpdate::Fn(Box::new(func))
    }
}

enum BatchCommand {
    Update(MetricSetUpdate),
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
                        match metrics {
                            MetricSetUpdate::Many(metrics) => {
                                for metric in metrics {
                                    set.add_or_update(metric);
                                }
                            }
                            MetricSetUpdate::Single(metric) => {
                                set.add_or_update(metric);
                            }
                            MetricSetUpdate::Fn(func) => {
                                func(&mut set);
                            }
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
    pub fn update(&self, update: impl Into<MetricSetUpdate>) {
        let update = update.into();
        self.sender.send(BatchCommand::Update(update)).unwrap();
    }

    #[inline]
    pub fn flush_into(&self, target: &mut MetricSet) {
        self.guard.wait();
        let set = self.set.lock().unwrap();
        set.flush_all_into(target);
    }

    #[inline]
    pub fn clear(&self) {
        self.guard.wait();
        let mut set = self.set.lock().unwrap();
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
