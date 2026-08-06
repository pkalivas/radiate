use crate::message::Warning;
use crate::{EventHandler, message::bus::EventCtx};
use crate::{LimitTriggered, message::EpochComplete};

const STAGNATION_WARNING_THRESHOLD: usize = 50;

#[derive(Clone)]
pub struct StagnationMonitorActor<T>(std::marker::PhantomData<T>);

impl<T> EventHandler<EpochComplete<T>> for StagnationMonitorActor<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: &EpochComplete<T>, ctx: &EventCtx) {
        let stag_count = message
            .metrics
            .stagnation_count()
            .map(|met| met.last_value() as usize)
            .unwrap_or_default();

        if stag_count >= STAGNATION_WARNING_THRESHOLD {
            ctx.publish(Warning(format!(
                "Stagnation detected: {} epochs without improvement",
                stag_count
            )));
        }
    }
}

impl<T> Default for StagnationMonitorActor<T> {
    fn default() -> Self {
        StagnationMonitorActor(std::marker::PhantomData)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogLevel {
    Info,
    Warn,
}

#[derive(Clone, Debug)]
pub struct LogEvent {
    pub level: LogLevel,
    pub index: Option<usize>,
    pub message: String,
}

#[derive(Clone, Default)]
pub struct LoggingHandler;

impl EventHandler<LogEvent> for LoggingHandler {
    fn handle(&mut self, message: &LogEvent, _: &EventCtx) {
        match message.level {
            LogLevel::Info => tracing::info!("{}", message.message),
            LogLevel::Warn => tracing::warn!("{}", message.message),
        }
    }
}

impl EventHandler<LimitTriggered> for LoggingHandler {
    fn handle(&mut self, message: &LimitTriggered, _: &EventCtx) {
        tracing::info!(
            "Limit triggered at generation {}: {:?}",
            message.0,
            message.1
        );
    }
}

impl EventHandler<Warning> for LoggingHandler {
    fn handle(&mut self, message: &Warning, _: &EventCtx) {
        tracing::warn!("{}", message.0);
    }
}

// impl MessageHandler<CheckpointSaved> for LoggingHandler {
//     fn handle(&mut self, message: CheckpointSaved, ctx: &Addr<Self>) {
//         ctx.send(LogEvent::Log(Log::info(
//             Some(message.index),
//             format!(
//                 "Checkpoint saved at index {}: {}",
//                 message.index, message.path
//             ),
//         )));
//     }
// }

// impl MessageHandler<ActorRegistered> for LoggingHandler {
//     fn handle(&mut self, message: ActorRegistered, ctx: &Addr<Self>) {
//         ctx.send(LogEvent::Log(Log::info(
//             None,
//             format!("Actor registered: {:?}", message.pid),
//         )));
//     }
// }

// impl MessageHandler<DeadLetter> for LoggingHandler {
//     fn handle(&mut self, message: DeadLetter, ctx: &Addr<Self>) {
//         ctx.send(LogEvent::Log(Log::warn(
//             None,
//             format!(
//                 "Dead letter received for actor {:?}: message type {}",
//                 message.pid, message.message_type
//             ),
//         )));
//     }
// }
