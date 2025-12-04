use crate::{Metric, MetricSet, WaitGroup, WaitGuard};
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

#[allow(dead_code)]
enum BatchCommand {
    Update((WaitGuard, MetricSetUpdate)),
    Complete,
}

pub struct BatchMetricUpdater {
    set: Arc<Mutex<MetricSet>>,
    thread: Option<JoinHandle<()>>,
    guard: Arc<WaitGroup>,
    sender: mpsc::Sender<BatchCommand>,
}

impl BatchMetricUpdater {
    #[allow(dead_code)]
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<BatchCommand>();

        let guard = Arc::new(WaitGroup::new());
        let receiver = Arc::new(Mutex::new(receiver));
        let set = Arc::new(Mutex::new(MetricSet::new()));
        let thread_set = Arc::clone(&set);
        let thread = std::thread::spawn(move || {
            loop {
                let updates = receiver.lock().unwrap().recv().unwrap();

                match updates {
                    BatchCommand::Update((waiter, metrics)) => {
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
                        drop(waiter);
                    }
                    BatchCommand::Complete => return,
                };
            }
        });

        BatchMetricUpdater {
            set,
            thread: Some(thread),
            guard: Arc::clone(&guard),
            sender,
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn update(&mut self, update: impl Into<MetricSetUpdate>) {
        self.sender
            .send(BatchCommand::Update((self.guard.guard(), update.into())))
            .unwrap();
    }

    #[allow(dead_code)]
    #[inline]
    pub fn flush_into(&mut self, target: &mut MetricSet) {
        self.guard.wait();
        let mut set = self.set.lock().unwrap();
        set.flush_all_into(target);
        set.clear();
    }
}

impl Drop for BatchMetricUpdater {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            self.guard.wait();
            self.sender.send(BatchCommand::Complete).unwrap();
            thread.join().unwrap();
        }
    }
}
