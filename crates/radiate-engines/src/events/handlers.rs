use crate::{Actor, ActorContext, MessageHandler, events::Warning};
use crate::{
    LimitTriggered,
    events::{EpochComplete, LimitProgress, Log},
};

const MAX_STAGNATION_EPOCHS: u64 = 2;

#[derive(Clone, Default)]
pub struct StagnationMonitorActor;

impl Actor for StagnationMonitorActor {}

impl<T> MessageHandler<EpochComplete<T>> for StagnationMonitorActor
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: EpochComplete<T>, ctx: &ActorContext) {
        let stag_count = message
            .metrics
            .stagnation_count()
            .map(|met| met.last_value() as u64)
            .unwrap_or_default();

        if stag_count >= MAX_STAGNATION_EPOCHS {
            ctx.send(Warning {
                index: message.index,
                message: format!(
                    "Stagnation detected: {} epochs without improvement",
                    stag_count
                ),
            });
        }
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

impl Actor for LoggingActor {
    fn on_init(&mut self, ctx: &ActorContext) {}
}

impl MessageHandler<LogEvent> for LoggingActor {
    fn handle(&mut self, message: LogEvent, _: &ActorContext) {
        match message {
            LogEvent::Log(event) => match event.level {
                crate::events::LogLevel::Info => tracing::info!("{}", event.message),
                crate::events::LogLevel::Warn => tracing::warn!("{}", event.message),
            },
            _ => {}
        }
    }
}

impl MessageHandler<LimitTriggered> for LoggingActor {
    fn handle(&mut self, message: LimitTriggered, ctx: &ActorContext) {
        ctx.send(LogEvent::Log(Log::info(
            Some(message.generation),
            message.description(),
        )));
    }
}

impl MessageHandler<Warning> for LoggingActor {
    fn handle(&mut self, message: Warning, ctx: &ActorContext) {
        ctx.send(LogEvent::Log(Log::warn(
            Some(message.index),
            message.message,
        )));
    }
}
