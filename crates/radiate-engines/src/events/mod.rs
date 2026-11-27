mod event;
mod handlers;
mod mailbox;
mod messages;
mod registry;
mod stats;
mod stream;
mod subscription;

#[cfg(feature = "serde")]
pub use handlers::CheckpointWriterHandler;
pub use handlers::{
    EngineLogger, HealthMonitor, LogEvent, LogLevel, LoggingHandler, StreamHandler,
};
pub use messages::{
    CheckpointSaved, EngineStart, EngineStateChange, EngineStop, EpochComplete, EpochStart,
    GenerationSnapshot, Improvement, LimitTriggered, StreamClosed, StreamEvent, StreamOpened,
    SubscriptionAdded, SubscriptionRemoved, Warning,
};

pub(crate) use event::EventScope;
pub use event::{Event, EventHandler, EventId, On};
pub use mailbox::{EventContext, MailBoxId};
pub use stats::{DeliveryMetrics, EventStats, StreamSnapshot, SubscriptionSnapshot};
pub use stream::EventStream;
pub use subscription::{Subscription, SubscriptionId};
