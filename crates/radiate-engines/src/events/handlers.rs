use crate::{Actor, ActorContext, EventHandler, MessageHandler, MetricSet};
use crate::{
    LimitTriggered,
    events::{EpochComplete, LimitProgress, Log},
};
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
///     .build();
///
/// engine.on::<EpochComplete<Vec<f32>>>(collector.clone());
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

    pub fn history(&self) -> Arc<Mutex<Vec<MetricSet>>> {
        Arc::clone(&self.history)
    }

    pub fn snapshot(&self) -> Vec<MetricSet> {
        self.history.lock().unwrap().clone()
    }
}

impl<T: Send + Sync + 'static> EventHandler<EpochComplete<T>> for MetricCollector {
    fn handle(&mut self, message: &EpochComplete<T>, _: &ActorContext) {
        self.history.lock().unwrap().push(message.metrics.clone());
    }
}

#[derive(Clone)]
pub enum LogEvent {
    LimitTriggered(LimitTriggered),
    LimitProgress(LimitProgress),
    Log(Log),
}

#[derive(Clone, Default)]
pub struct LoggingActor;

impl Actor for LoggingActor {}

impl MessageHandler<LogEvent> for LoggingActor {
    fn handle(&mut self, message: LogEvent, _: &ActorContext) {
        match message {
            LogEvent::LimitTriggered(event) => {
                tracing::info!("Limit triggered: {} - {}", event.kind, event.description);
            }
            LogEvent::LimitProgress(event) => {
                tracing::info!("Limit progress: {}", event.description(),);
            }
            LogEvent::Log(event) => match event.level {
                crate::events::LogLevel::Info => tracing::info!("{}", event.message),
                crate::events::LogLevel::Warn => tracing::warn!("{}", event.message),
            },
        }
    }
}
