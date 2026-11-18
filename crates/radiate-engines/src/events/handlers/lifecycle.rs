use crate::{
    EventContext, EventHandler, Handler, LogEvent, LogLevel,
    events::messages::{BuilderEvent, DeferredEvent, StreamEvent},
};
use radiate_core::error::RadiateResult;

pub struct LifecycleHandler;

impl EventHandler for LifecycleHandler {
    fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
        ctx.subscribe::<DeferredEvent<BuilderEvent>>();
        ctx.subscribe::<DeferredEvent<StreamEvent>>();
        ctx.subscribe::<StreamEvent>();
        Ok(())
    }
}

impl Handler<DeferredEvent<BuilderEvent>> for LifecycleHandler {
    fn handle(&mut self, event: &DeferredEvent<BuilderEvent>, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(LogLevel::Info, event.0.0.clone()));
    }
}

impl Handler<DeferredEvent<StreamEvent>> for LifecycleHandler {
    fn handle(&mut self, event: &DeferredEvent<StreamEvent>, ctx: &EventContext<'_, Self>) {
        match &event.0 {
            StreamEvent::Start => ctx.publish(LogEvent(LogLevel::Info, "Stream started".into())),
            StreamEvent::Stop => ctx.publish(LogEvent(LogLevel::Info, "Stream stopped".into())),
            StreamEvent::SubscriptionAdded(sub) => ctx.publish(LogEvent(
                LogLevel::Info,
                format!("Subscription added: {:?}", sub),
            )),
        }
    }
}

impl Handler<StreamEvent> for LifecycleHandler {
    fn handle(&mut self, event: &StreamEvent, ctx: &EventContext<'_, Self>) {
        match event {
            StreamEvent::Start => ctx.publish(LogEvent(LogLevel::Info, "Stream started".into())),
            StreamEvent::Stop => ctx.publish(LogEvent(LogLevel::Info, "Stream stopped".into())),
            StreamEvent::SubscriptionAdded(sub) => ctx.publish(LogEvent(
                LogLevel::Info,
                format!("Subscription added: {:?}", sub),
            )),
        }
    }
}
