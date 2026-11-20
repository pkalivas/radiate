use crate::{
    EventContext, EventHandler, Handler, LogEvent, LogLevel,
    events::messages::{BuilderEvent, DeferredEvent, SubscriptionEvent},
};
use radiate_core::error::RadiateResult;

pub struct LifecycleHandler;

impl EventHandler for LifecycleHandler {
    fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
        ctx.subscribe::<DeferredEvent<BuilderEvent>>();
        ctx.subscribe::<DeferredEvent<SubscriptionEvent>>();

        Ok(())
    }
}

impl Handler<DeferredEvent<BuilderEvent>> for LifecycleHandler {
    fn handle(&mut self, event: &DeferredEvent<BuilderEvent>, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(LogLevel::Info, event.0.0.clone()));
    }
}

impl Handler<DeferredEvent<SubscriptionEvent>> for LifecycleHandler {
    fn handle(&mut self, event: &DeferredEvent<SubscriptionEvent>, ctx: &EventContext<'_, Self>) {
        match &event.0 {
            SubscriptionEvent::Added(sub) => {
                ctx.publish(LogEvent(
                    LogLevel::Info,
                    format!(
                        "Subscription | Added   | {:?}",
                        sub.name().unwrap_or_default()
                    ),
                ));
            }
            SubscriptionEvent::Removed(stats) => {
                ctx.publish(LogEvent(
                    LogLevel::Info,
                    format!(
                        "Subscription | Removed | {:?}",
                        stats.name.clone().unwrap_or_default()
                    ),
                ));
            }
        }
    }
}
