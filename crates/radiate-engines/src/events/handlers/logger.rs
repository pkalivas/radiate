use crate::{
    EventHandler,
    events::{CheckpointSaved, EngineStateChange, EventContext, Subscriber, Subscribes, Warning},
};
use crate::{LimitTriggered, events::EpochComplete};
use radiate_core::Objective;
use std::marker::PhantomData;

pub struct EngineLogger<T> {
    _marker: PhantomData<T>,
}

impl<T> Default for EngineLogger<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> EngineLogger<T> {
    pub fn new() -> Self {
        EngineLogger {
            _marker: PhantomData,
        }
    }
}

impl<T> Subscribes for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn subscribe(subscriber: &Subscriber<Self>) {
        subscriber.subscribe::<LimitTriggered>();
        subscriber.subscribe::<Warning>();
        subscriber.subscribe::<CheckpointSaved>();
        subscriber.subscribe::<EpochComplete<T>>();
        subscriber.subscribe::<EngineStateChange>();
    }
}

impl<T> EventHandler<LimitTriggered> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: &LimitTriggered, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!("Limit triggered: {:?}", message.1),
        ));
    }
}

impl<T> EventHandler<Warning> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: &Warning, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(LogLevel::Warn, message.0.clone()));
    }
}

impl<T> EventHandler<CheckpointSaved> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: &CheckpointSaved, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!(
                "Checkpoint saved at index {}: {}",
                message.index, message.path
            ),
        ));
    }
}

impl<T> EventHandler<EpochComplete<T>> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, event: &EpochComplete<T>, ctx: &EventContext<'_, Self>) {
        let time = event
            .metrics
            .time()
            .and_then(|m| m.times().map(|t| t.sum()))
            .unwrap_or_default();

        let msg = match event.objective {
            Objective::Single(_) => format!(
                "Epoch {:<4} | Score: {:>8.4} | Time: {:>5.2?}",
                event.index,
                event.score.as_f32(),
                time
            ),
            Objective::Multi(_) => {
                let front_size = event
                    .metrics
                    .front_size()
                    .map(|ent| ent.last_value())
                    .unwrap_or(0.0);
                format!(
                    "Epoch {:<4} | Front Size: {:.3} | Time: {:>5.2?}",
                    event.index, front_size, time
                )
            }
        };

        ctx.publish(LogEvent(LogLevel::Info, msg));
    }
}

impl<T> EventHandler<EngineStateChange> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, event: &EngineStateChange, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!("State Change: {:?} -> {:?}", event.from, event.to),
        ));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogLevel {
    Info,
    Warn,
}

#[derive(Clone, Debug)]
pub struct LogEvent(pub LogLevel, pub String);

#[derive(Clone, Default)]
pub struct LoggingHandler;

impl EventHandler<LogEvent> for LoggingHandler {
    fn handle(&mut self, message: &LogEvent, _: &EventContext<'_, Self>) {
        match message.0 {
            LogLevel::Info => tracing::info!("{}", message.1),
            LogLevel::Warn => tracing::warn!("{}", message.1),
        }
    }
}
