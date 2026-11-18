mod handlers;
mod messages;
mod stream;
mod subscriber;
mod subscription;

#[cfg(feature = "serde")]
pub use handlers::CheckpointWriterHandler;
pub use handlers::{
    EngineLogger, HealthMonitor, LifecycleHandler, LogEvent, LogLevel, LoggingHandler,
};
pub use messages::{
    BuilderEvent, CheckpointSaved, DeferredEvent, EngineStart, EngineStateChange, EngineStop, EpochComplete,
    EpochStart, GenerationSnapshot, Improvement, LimitTriggered, Warning,
};
pub use stream::EventStream;
pub use subscriber::{Event, EventContext, EventHandler, EventId, Handler, Subscriber};
pub use subscription::{Subscription, SubscriptionId, SubscriptionInfo, SubscriptionStats};

pub trait Publish<T> {
    fn publish(&self, event: T);
}
