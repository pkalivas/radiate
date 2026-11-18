use crate::{
    EventContext, EventHandler, Handler, LogEvent, LogLevel,
    events::messages::{BuilderEvent, DeferredEvent},
};
use radiate_core::error::RadiateResult;

const SUBSCRIPTION_NAME: &str = "LifecycleHandler";

pub struct LifecycleHandler;

impl EventHandler for LifecycleHandler {
    fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
        ctx.subscribe::<DeferredEvent<BuilderEvent>>()
            .named(SUBSCRIPTION_NAME);

        Ok(())
    }
}

impl Handler<DeferredEvent<BuilderEvent>> for LifecycleHandler {
    fn handle(&mut self, event: &DeferredEvent<BuilderEvent>, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(LogLevel::Info, event.0.0.clone()));
    }
}
