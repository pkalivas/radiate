use crate::{
    Actor, EngineStop, EventHandler,
    events::{
        CheckpointSaved, EngineStart, EngineStateChange, Message, MessageHandler, StreamEvent,
        Warning, addr::ActorContext,
    },
};
use crate::{LimitTriggered, events::EpochComplete};
use radiate_core::{EngineState, Objective};
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

impl<T> Actor for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn name(&self) -> &str {
        "EngineLogger"
    }

    fn started(&mut self, ctx: &ActorContext<Self>)
    where
        Self: Sized,
    {
        ctx.subscribe::<LimitTriggered>();
        ctx.subscribe::<Warning>();
        ctx.subscribe::<CheckpointSaved>();
        ctx.subscribe::<EpochComplete<T>>();
        ctx.subscribe::<EngineState>();
        ctx.subscribe::<EngineStop<T>>();
        ctx.subscribe::<EngineStart>();
        ctx.subscribe::<EngineStateChange>();
        ctx.subscribe::<StreamEvent>();
    }
}

impl<T> MessageHandler<StreamEvent> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: &StreamEvent, ctx: &ActorContext<Self>) {
        match message {
            StreamEvent::HandlerRegistered(name, id) => {
                let actor_id = format!("{}-{:?}", name, id.get());
                ctx.publish(LogEvent(LogLevel::Info, format!("{} Registered", actor_id)));
            }
            StreamEvent::SubscriptionAdded(name, actor_id, subscription_id) => {
                let actor_id = format!("{}-{:?}", name, actor_id.get());
                ctx.publish(LogEvent(
                    LogLevel::Info,
                    format!("{} New Subscription added: {:?}", actor_id, subscription_id),
                ));
            }
            StreamEvent::FnHandler(subscription_id) => {
                ctx.publish(LogEvent(
                    LogLevel::Info,
                    format!("Function handler added: {:?}", subscription_id),
                ));
            }
        }
    }
}

impl<T> MessageHandler<LimitTriggered> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: &LimitTriggered, ctx: &ActorContext<Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!("Limit triggered: {:?}", message.1),
        ));
    }
}

impl<T> MessageHandler<Warning> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: &Warning, ctx: &ActorContext<Self>) {
        ctx.publish(LogEvent(LogLevel::Warn, message.0.clone()));
    }
}

impl<T> MessageHandler<CheckpointSaved> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, message: &CheckpointSaved, ctx: &ActorContext<Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!(
                "Checkpoint saved at index {}: {}",
                message.index, message.path
            ),
        ));
    }
}

impl<T> MessageHandler<EpochComplete<T>> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, event: &EpochComplete<T>, ctx: &ActorContext<Self>) {
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

impl<T> MessageHandler<EngineState> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, event: &EngineState, ctx: &ActorContext<Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!("State Change: {:?}", *event),
        ));
    }
}

impl<T> MessageHandler<EngineStateChange> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, event: &EngineStateChange, ctx: &ActorContext<Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!("State Change: {:?} -> {:?}", event.from, event.to),
        ));
    }
}

impl<T> MessageHandler<EngineStart> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, _: &EngineStart, ctx: &ActorContext<Self>) {
        ctx.publish(EngineState::Running);
    }
}

impl<T> MessageHandler<EngineStop<T>> for EngineLogger<T>
where
    T: Send + Sync + 'static,
{
    fn handle(&mut self, _: &EngineStop<T>, ctx: &ActorContext<Self>) {
        ctx.publish(EngineState::Stopped);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogLevel {
    Info,
    Warn,
}

#[derive(Clone, Debug)]
pub struct LogEvent(pub LogLevel, pub String);

impl Message for LogEvent {
    type Response = ();
}

#[derive(Clone, Default)]
pub struct LoggingHandler;

impl EventHandler<LogEvent> for LoggingHandler {
    fn handle(&mut self, message: &LogEvent) {
        match message.0 {
            LogLevel::Info => tracing::info!("{}", message.1),
            LogLevel::Warn => tracing::warn!("{}", message.1),
        }
    }
}
