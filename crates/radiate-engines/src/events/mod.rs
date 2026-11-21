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
    EngineLogger, HealthMonitor, LogEvent, LogLevel, LoggingHandler, StreamHandler, StreamMonitor,
};
pub use messages::{
    CheckpointSaved, EngineStart, EngineStateChange, EngineStop, EpochComplete, EpochStart,
    GenerationSnapshot, Improvement, LimitTriggered, StreamClosed, StreamEvent, StreamOpened,
    SubscriptionAdded, SubscriptionRemoved, Warning,
};

pub(crate) use event::EventScope;
pub use event::{Event, EventHandler, EventId, Handler};
pub use mailbox::{EventContext, MailBoxId};
pub use stats::{EventStats, StreamSnapshot, SubscriptionInfo, SubscriptionStats};
pub use stream::EventStream;
pub use subscription::{Subscription, SubscriptionId};
