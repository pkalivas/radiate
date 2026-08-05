use crate::{
    Actor, Addr, Log, MessageHandler,
    actors::{ActorRegistered, DeadLetter},
    events::{CheckpointSaved, Warning},
};
use crate::{
    LimitTriggered,
    events::{EpochComplete, LimitProgress},
};

const STAGNATION_WARNING_THRESHOLD: usize = 50;

#[derive(Clone, Default)]
pub struct StagnationMonitorActor<T> {
    pub _marker: std::marker::PhantomData<T>,
}

impl<T> Actor for StagnationMonitorActor<T>
where
    T: Send + Sync + 'static,
{
    fn on_init(&mut self, addr: &Addr<Self>) {
        addr.subscribe::<EpochComplete<T>>();
    }
}

impl<T> MessageHandler<EpochComplete<T>> for StagnationMonitorActor<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: EpochComplete<T>, ctx: &Addr<Self>) {
        let stag_count = message
            .metrics
            .stagnation_count()
            .map(|met| met.last_value() as usize)
            .unwrap_or_default();

        if stag_count >= STAGNATION_WARNING_THRESHOLD {
            ctx.publish(Warning {
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
    fn on_init(&mut self, addr: &Addr<Self>) {
        addr.subscribe::<LimitTriggered>();
        addr.subscribe::<Warning>();
        addr.subscribe::<LogEvent>();
        addr.subscribe::<CheckpointSaved>();
        addr.subscribe::<ActorRegistered>();
        addr.subscribe::<DeadLetter>();
    }
}

impl MessageHandler<LogEvent> for LoggingActor {
    fn handle(&mut self, message: LogEvent, _: &Addr<Self>) {
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
    fn handle(&mut self, message: LimitTriggered, ctx: &Addr<Self>) {
        ctx.send(LogEvent::Log(Log::info(
            Some(message.generation),
            message.description(),
        )));
    }
}

impl MessageHandler<Warning> for LoggingActor {
    fn handle(&mut self, message: Warning, ctx: &Addr<Self>) {
        ctx.send(LogEvent::Log(Log::warn(
            Some(message.index),
            message.message,
        )));
    }
}

impl MessageHandler<CheckpointSaved> for LoggingActor {
    fn handle(&mut self, message: CheckpointSaved, ctx: &Addr<Self>) {
        ctx.send(LogEvent::Log(Log::info(
            Some(message.index),
            format!(
                "Checkpoint saved at index {}: {}",
                message.index, message.path
            ),
        )));
    }
}

impl MessageHandler<ActorRegistered> for LoggingActor {
    fn handle(&mut self, message: ActorRegistered, ctx: &Addr<Self>) {
        ctx.send(LogEvent::Log(Log::info(
            None,
            format!("Actor registered: {:?}", message.pid),
        )));
    }
}

impl MessageHandler<DeadLetter> for LoggingActor {
    fn handle(&mut self, message: DeadLetter, ctx: &Addr<Self>) {
        ctx.send(LogEvent::Log(Log::warn(
            None,
            format!(
                "Dead letter received for actor {:?}: message type {}",
                message.pid, message.message_type
            ),
        )));
    }
}
