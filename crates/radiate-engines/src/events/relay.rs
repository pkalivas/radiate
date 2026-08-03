use crate::events::{
    EngineEvent, EngineImproved, EngineStart, EngineStopped, EpochCompleted, EpochStart,
    LimitTriggered, LogEvent,
};
use radiate_core::{EventContext, MessageBroker};

/// The fan-in: one subscription per concrete engine-event type, each just
/// re-wrapping its message into `EngineEvent<T>` and re-publishing it on
/// the same system. This is what lets `EngineEvent<T>` stay the single
/// "catch everything" type without any dispatch call site (`engine.rs`,
/// `limit.rs`, ...) needing to know the wildcard exists — they only ever
/// send their own concrete type, same as `LimitTriggered` already does.
///
/// Only worth paying for if something actually wants the aggregate view —
/// call once, gated on `has_subscribers::<EngineEvent<T>>()`, from wherever
/// the event system is finalized at build time. Not per-message.
pub(crate) fn event_relay<T>(system: &MessageBroker)
where
    T: Clone + Send + Sync + 'static,
{
    system
        .on::<EngineStart>()
        .handle(|msg: &EngineStart, ctx: &EventContext| {
            ctx.send(EngineEvent::<T>::Started(msg.clone()));
        });

    system
        .on::<EpochStart>()
        .handle(|msg: &EpochStart, ctx: &EventContext| {
            ctx.send(EngineEvent::<T>::EpochStarted(msg.clone()));
        });

    system
        .on::<EngineImproved<T>>()
        .handle(|msg: &EngineImproved<T>, ctx: &EventContext| {
            ctx.send(EngineEvent::<T>::Improved(msg.clone()));
        });

    system
        .on::<EpochCompleted<T>>()
        .handle(|msg: &EpochCompleted<T>, ctx: &EventContext| {
            ctx.send(EngineEvent::<T>::EpochCompleted(msg.clone()));
        });

    system
        .on::<EngineStopped<T>>()
        .handle(|msg: &EngineStopped<T>, ctx: &EventContext| {
            ctx.send(EngineEvent::<T>::Stopped(msg.clone()));
        });

    system
        .on::<LimitTriggered>()
        .handle(|msg: &LimitTriggered, ctx: &EventContext| {
            ctx.send(EngineEvent::<T>::LimitTriggered(msg.clone()));
        });

    system
        .on::<LogEvent>()
        .handle(|msg: &LogEvent, ctx: &EventContext| {
            ctx.send(EngineEvent::<T>::Log(msg.clone()));
        });
}
