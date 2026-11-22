use crate::{
    Event, On, Warning,
    events::{EngineStateChange, EventContext, EventHandler},
};
use crate::{LimitTriggered, events::EpochComplete};
use radiate_core::Objective;
use radiate_core::error::RadiateResult;
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

impl<T> EventHandler for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
        ctx.subscribe::<LimitTriggered>();
        ctx.subscribe::<EpochComplete<T>>();
        ctx.subscribe::<EngineStateChange>();
        ctx.subscribe::<Warning>();
        Ok(())
    }
}

impl<T> On<LimitTriggered> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: &LimitTriggered, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            radiate_utils::three_col_format(
                "Engine",
                "Limit Triggered",
                &format!("{:?}", message.1),
            ),
        ))
        .unwrap();
    }
}

impl<T> On<EpochComplete<T>> for EngineLogger<T>
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

        ctx.publish(LogEvent(LogLevel::Info, msg)).unwrap();
    }
}

impl<T> On<EngineStateChange> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, event: &EngineStateChange, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            radiate_utils::three_col_format(
                "Engine",
                "State Change",
                &format!("{:?} -> {:?}", event.from, event.to),
            ),
        ))
        .unwrap();
    }
}

impl<T> On<Warning> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, event: &Warning, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(LogLevel::Warn, event.0.clone()))
            .unwrap();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogLevel {
    Info,
    Warn,
}

#[derive(Clone, Debug)]
pub struct LogEvent(pub LogLevel, pub String);

impl Event for LogEvent {}

#[derive(Clone, Default)]
pub struct LoggingHandler(bool);

impl LoggingHandler {
    pub fn new() -> Self {
        Self(crate::logging_enabled())
    }
}

impl EventHandler for LoggingHandler {
    fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
        ctx.subscribe::<LogEvent>();
        Ok(())
    }
}

impl On<LogEvent> for LoggingHandler {
    fn handle(&mut self, message: &LogEvent, _ctx: &EventContext<'_, Self>) {
        if !self.0 {
            return;
        }

        match message.0 {
            LogLevel::Info => tracing::info!("{}", message.1),
            LogLevel::Warn => tracing::warn!("{}", message.1),
        }
    }
}
