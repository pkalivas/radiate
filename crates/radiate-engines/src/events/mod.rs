mod handlers;
mod mailbox;
mod messages;
mod stats;
mod stream;
mod subscription;

#[cfg(feature = "serde")]
pub use handlers::CheckpointWriterHandler;
pub use handlers::{
    EngineLogger, HealthMonitor, LifecycleHandler, LogEvent, LogLevel, LoggingHandler,
};
pub use messages::{
    BuilderEvent, CheckpointSaved, DeferredEvent, EngineStart, EngineStateChange, EngineStop,
    EpochComplete, EpochStart, GenerationSnapshot, Improvement, LimitTriggered, Warning,
};
use radiate_core::{SmallStr, error::RadiateResult};
use radiate_utils::{generate_metric_key, sentry_id};

pub use mailbox::EventContext;
pub use stats::{StreamStats, SubscriptionInfo, SubscriptionStats};
pub use stream::EventStream;
pub use subscription::{Subscription, SubscriptionId};

const EVENT_PREFIX: &str = "event";

sentry_id!(EventId);

pub trait Event: Send + Sync + 'static {
    fn name() -> SmallStr {
        SmallStr::from_string(generate_metric_key::<Self>(EVENT_PREFIX))
    }
}

pub trait Handler<E: Event>: Send + 'static {
    fn handle(&mut self, event: &E, ctx: &EventContext<'_, Self>)
    where
        Self: Sized;
}

pub trait EventHandler: Send + 'static {
    fn start(&mut self, _ctx: &EventContext<'_, Self>) -> RadiateResult<()>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl<E, F> Handler<E> for F
where
    E: Event,
    F: FnMut(&E) + Send + 'static,
{
    fn handle(&mut self, event: &E, _ctx: &EventContext<'_, Self>) {
        self(event)
    }
}
