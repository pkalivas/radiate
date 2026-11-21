use crate::{
    EventContext, EventHandler, Handler, LogEvent, LogLevel, StreamClosed, StreamEvent,
    StreamOpened, SubscriptionAdded, SubscriptionRemoved,
};
use radiate_core::error::RadiateResult;

pub struct StreamHandler;

impl EventHandler for StreamHandler {
    fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
        ctx.subscribe::<StreamEvent<SubscriptionAdded>>();
        ctx.subscribe::<StreamEvent<SubscriptionRemoved>>();
        ctx.subscribe::<StreamEvent<StreamOpened>>();
        ctx.subscribe::<StreamEvent<StreamClosed>>();

        Ok(())
    }
}

impl Handler<StreamEvent<StreamOpened>> for StreamHandler {
    fn handle(&mut self, _: &StreamEvent<StreamOpened>, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            radiate_utils::three_col_format("Stream", "Opened", ""),
        ));
    }
}

impl Handler<StreamEvent<StreamClosed>> for StreamHandler {
    fn handle(&mut self, _: &StreamEvent<StreamClosed>, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            radiate_utils::three_col_format("Stream", "Closed", ""),
        ));
    }
}

impl Handler<StreamEvent<SubscriptionAdded>> for StreamHandler {
    fn handle(&mut self, event: &StreamEvent<SubscriptionAdded>, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            radiate_utils::three_col_format("Sub", "Added", &event.0.0.name().unwrap_or_default()),
        ));
    }
}

impl Handler<StreamEvent<SubscriptionRemoved>> for StreamHandler {
    fn handle(&mut self, event: &StreamEvent<SubscriptionRemoved>, ctx: &EventContext<'_, Self>) {
        ctx.publish(LogEvent(
            LogLevel::Info,
            radiate_utils::three_col_format(
                "Sub",
                "Removed",
                &event.0.0.name().unwrap_or_default(),
            ),
        ));
    }
}
