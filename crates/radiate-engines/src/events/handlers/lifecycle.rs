use crate::{
    EventContext, EventHandler, Handler, LogEvent, LogLevel, StreamClosed, StreamEvent,
    StreamOpened, SubscriptionAdded, SubscriptionRemoved,
};
use radiate_core::error::RadiateResult;

pub struct LifecycleHandler;

impl EventHandler for LifecycleHandler {
    fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
        ctx.subscribe::<StreamEvent<SubscriptionAdded>>();
        ctx.subscribe::<StreamEvent<SubscriptionRemoved>>();
        ctx.subscribe::<StreamEvent<StreamOpened>>();
        ctx.subscribe::<StreamEvent<StreamClosed>>();

        Ok(())
    }
}

impl Handler<StreamEvent<StreamOpened>> for LifecycleHandler {
    fn handle(&mut self, _: &StreamEvent<StreamOpened>, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(LogLevel::Info, format!("Stream | Opened")));
    }
}

impl Handler<StreamEvent<StreamClosed>> for LifecycleHandler {
    fn handle(&mut self, _: &StreamEvent<StreamClosed>, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(LogLevel::Info, format!("Stream | Closed")));
    }
}

impl Handler<StreamEvent<SubscriptionAdded>> for LifecycleHandler {
    fn handle(&mut self, event: &StreamEvent<SubscriptionAdded>, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!(
                "Subscription | Added   | {:?}",
                event.0.0.name().unwrap_or_default()
            ),
        ));
    }
}

impl Handler<StreamEvent<SubscriptionRemoved>> for LifecycleHandler {
    fn handle(&mut self, event: &StreamEvent<SubscriptionRemoved>, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            format!(
                "Subscription | Removed | {:?}",
                event.0.0.name().unwrap_or_default()
            ),
        ));
    }
}
