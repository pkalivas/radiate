use crate::{
    EventContext, EventHandler, Handler, LogEvent, LogLevel,
    events::messages::{StreamDrained, SubscriptionAdded, SubscriptionRemoved},
};
use radiate_core::error::RadiateResult;

pub struct LifecycleHandler;

impl EventHandler for LifecycleHandler {
    fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
        ctx.subscribe::<SubscriptionAdded>();
        ctx.subscribe::<SubscriptionRemoved>();
        ctx.subscribe::<StreamDrained>();

        Ok(())
    }
}

impl Handler<SubscriptionAdded> for LifecycleHandler {
    fn handle(&mut self, event: &SubscriptionAdded, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!(
                "Subscription | Added   | {:?}",
                event.0.name().unwrap_or_default()
            ),
        ));
    }
}

impl Handler<SubscriptionRemoved> for LifecycleHandler {
    fn handle(&mut self, event: &SubscriptionRemoved, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!(
                "Subscription | Removed | {:?}",
                event.0.name.clone().unwrap_or_default()
            ),
        ));
    }
}

impl Handler<StreamDrained> for LifecycleHandler {
    fn handle(&mut self, _event: &StreamDrained, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(LogLevel::Info, "Stream drained".to_string()));
    }
}
