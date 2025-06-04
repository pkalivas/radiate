use super::{EngineEvent, events::Event};
use radiate_core::MetricSet;
use std::{
    fmt::{Debug, Display},
    sync::{Arc, RwLock, RwLockReadGuard},
};

pub trait EventHandler<T>: Send + Sync {
    fn handle(&mut self, event: Event<T>);
}

impl<T, F> EventHandler<T> for F
where
    F: Fn(Event<T>) + Send + Sync + 'static,
{
    fn handle(&mut self, event: Event<T>) {
        (self)(event)
    }
}

pub struct EventLogger {
    full: bool,
}

impl EventLogger {
    pub fn new(full: bool) -> Self {
        EventLogger { full }
    }
}

impl<T: Debug + Display> EventHandler<T> for EventLogger {
    fn handle(&mut self, event: Event<T>) {
        if self.full {
            println!("Event: {:?}", event);
        } else {
            println!("Event: [{:?}]: {}", event.id(), event.data());
        }
    }
}

impl Default for EventLogger {
    fn default() -> Self {
        EventLogger::new(false)
    }
}

#[derive(Default, Clone)]
pub struct MetricsAggregator {
    metrics: Arc<RwLock<Vec<MetricSet>>>,
}

impl MetricsAggregator {
    pub fn new() -> Self {
        MetricsAggregator {
            metrics: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn metrics(&self) -> RwLockReadGuard<Vec<MetricSet>> {
        self.metrics.read().unwrap()
    }

    pub fn aggregate(&self) -> MetricSet {
        self.metrics
            .read()
            .unwrap()
            .iter()
            .fold(MetricSet::default(), |mut acc, m| {
                acc.merge(m);
                acc
            })
    }
}

impl<T> EventHandler<EngineEvent<T>> for MetricsAggregator {
    fn handle(&mut self, event: Event<EngineEvent<T>>) {
        if let EngineEvent::EpochComplete { metrics, .. } = event.data() {
            self.metrics.write().unwrap().push(metrics.clone());
        }
    }
}
