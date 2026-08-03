use radiate_core::{EventContext, EventSystem};

use crate::events::{
    EngineEvent, EngineImproved, EngineStart, EngineStopped, EpochCompleted, EpochStart,
    LimitTriggered, LogInfo, LogWarn,
};

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
pub(crate) fn event_relay<T>(system: &EventSystem)
where
    T: Clone + Send + Sync + 'static,
{
    system.subscribe::<EngineStart, _>(|msg: EngineStart, ctx: &EventContext| {
        ctx.send(EngineEvent::<T>::Started(msg));
    });

    system.subscribe::<EpochStart, _>(|msg: EpochStart, ctx: &EventContext| {
        ctx.send(EngineEvent::<T>::EpochStarted(msg));
    });

    system.subscribe::<EngineImproved<T>, _>(|msg: EngineImproved<T>, ctx: &EventContext| {
        ctx.send(EngineEvent::Improved(msg));
    });

    system.subscribe::<EpochCompleted<T>, _>(|msg: EpochCompleted<T>, ctx: &EventContext| {
        ctx.send(EngineEvent::EpochCompleted(msg));
    });

    system.subscribe::<EngineStopped<T>, _>(|msg: EngineStopped<T>, ctx: &EventContext| {
        ctx.send(EngineEvent::Stopped(msg));
    });

    system.subscribe::<LimitTriggered, _>(|msg: LimitTriggered, ctx: &EventContext| {
        ctx.send(EngineEvent::<T>::LimitTriggered(msg));
    });

    system.subscribe::<LogInfo, _>(|msg: LogInfo, ctx: &EventContext| {
        ctx.send(EngineEvent::<T>::LogInfo(msg.0));
    });

    system.subscribe::<LogWarn, _>(|msg: LogWarn, ctx: &EventContext| {
        ctx.send(EngineEvent::<T>::LogWarn(msg.0))
    });
}
