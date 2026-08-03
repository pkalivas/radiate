use crate::events::EpochCompleted;
use radiate_core::{EventContext, EventHandler, MetricSet};
use std::sync::{Arc, Mutex};

/// Collects the `MetricSet` from every completed generation into a shared,
/// externally-readable history. State comes out of an actor the same way it
/// goes in: `history()` hands back a clone of the `Arc`, so the caller keeps
/// a live handle to the same storage the handler is writing to — readable
/// mid-run, not just after the engine stops.
///
/// ```
/// # use radiate_engines::*;
/// # use radiate_core::*;
/// let collector = MetricCollector::new();
/// let history = collector.history();
///
/// let engine = GeneticEngine::builder()
///     .codec(FloatCodec::vector(4, -5.0..5.0))
///     .fitness_fn(|geno: Vec<f32>| geno.iter().sum::<f32>())
///     .on_epoch_complete(collector)
///     .build();
///
/// let result = engine.run(|epoch| epoch.index() >= 5);
/// assert_eq!(history.lock().unwrap().len(), 5);
/// ```
#[derive(Clone, Default)]
pub struct MetricCollector {
    history: Arc<Mutex<Vec<MetricSet>>>,
}

impl MetricCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// A shared handle to the collected history — the same storage the
    /// handler writes to, not a snapshot. Safe to read while the engine is
    /// still running.
    pub fn history(&self) -> Arc<Mutex<Vec<MetricSet>>> {
        Arc::clone(&self.history)
    }

    /// Convenience for the common case: a point-in-time copy instead of the
    /// raw shared handle.
    pub fn snapshot(&self) -> Vec<MetricSet> {
        self.history.lock().unwrap().clone()
    }
}

impl<T: Send + Sync + 'static> EventHandler<EpochCompleted<T>> for MetricCollector {
    fn handle(&mut self, message: EpochCompleted<T>, _ctx: &EventContext) {
        self.history.lock().unwrap().push(message.metrics.clone());
    }
}

#[derive(Clone)]
pub struct Info(pub String);

#[derive(Clone)]
pub struct Error(pub String);

#[derive(Clone)]
pub struct Warn(pub String);

#[derive(Clone)]
pub struct Debug(pub String);

#[derive(Clone, Default)]
pub struct LoggingHandler;

impl EventHandler<Info> for LoggingHandler {
    fn handle(&mut self, message: Info, _ctx: &EventContext) {
        tracing::info!("{}", message.0);
    }
}

impl EventHandler<Error> for LoggingHandler {
    fn handle(&mut self, message: Error, _ctx: &EventContext) {
        tracing::error!("{}", message.0);
    }
}

impl EventHandler<Warn> for LoggingHandler {
    fn handle(&mut self, message: Warn, _ctx: &EventContext) {
        tracing::warn!("{}", message.0);
    }
}

impl EventHandler<Debug> for LoggingHandler {
    fn handle(&mut self, message: Debug, _ctx: &EventContext) {
        tracing::debug!("{}", message.0);
    }
}
