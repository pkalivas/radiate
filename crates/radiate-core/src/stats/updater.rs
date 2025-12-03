use crate::{Metric, MetricSet, MetricUpdate, WaitGroup, WaitGuard};
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
    Update((WaitGuard, MetricSetUpdate)),
    Complete,
    Clear,
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
                    BatchCommand::Clear => {
                        let mut set = thread_set.lock().unwrap();
                        set.clear();
                    }
                    BatchCommand::Complete => return,
                };
            }
        });

        BatchMetricUpdater {
            set,
            thread: Some(thread),
            guard,
            sender,
        }
    }

    pub fn upsert<'a>(&self, name: &'static str, update: impl Into<MetricUpdate<'static>>) {
        let update = update.into();
        self.sender
            .send(BatchCommand::Update((
                self.guard.guard(),
                MetricSetUpdate::Fn(Box::new(move |set: &mut MetricSet| {
                    set.upsert(name, update);
                })),
            )))
            .unwrap();
    }

    #[inline]
    pub fn update(&self, update: impl Into<MetricSetUpdate>) {
        self.sender
            .send(BatchCommand::Update((self.guard.guard(), update.into())))
            .unwrap();
    }

    #[inline]
    pub fn flush_into(&self, target: &mut MetricSet) {
        {
            self.guard.wait();
            let set = self.set.lock().unwrap();
            set.flush_all_into(target);
        }

        self.sender.send(BatchCommand::Clear).unwrap();
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
            self.guard.wait();
            let _ = self.sender.send(BatchCommand::Complete);
            let _ = thread.join();
        }
    }
}
