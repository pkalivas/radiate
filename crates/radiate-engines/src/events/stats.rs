use crate::{MailBoxId, SubscriptionId};
use radiate_core::SmallStr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StreamSnapshot {
    /// When this snapshot was taken, as time since the stream was created.
    /// Taken by the publisher, so it stays accurate however late a handler
    /// on a worker pool gets to it.
    pub at: Duration,
    pub event_count: usize,
    pub events: Vec<EventStats>,
    pub subscribers: Vec<SubscriptionInfo>,
}

/// How often one event type has been published, from the time anything
/// first subscribed to it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct EventStats {
    pub event: SmallStr,
    /// Built and dispatched to its subscriptions.
    pub published: u64,
    /// Lazy publishes that were never built because no subscription was due.
    pub unbuilt: u64,
    /// Published, but every subscription skipped it on its schedule.
    pub undelivered: u64,
}

/// A point-in-time copy of a [`Subscription`]'s delivery counters.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SubscriptionStats {
    pub delivered: u64,
    pub skipped: u64,
    pub handled: u64,
}

impl SubscriptionStats {
    /// Events delivered but not yet handled. Always zero on a serial executor;
    /// on a worker pool this is how far the handler is behind the engine.
    pub fn queued(&self) -> u64 {
        self.delivered.saturating_sub(self.handled)
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SubscriptionInfo {
    pub id: SubscriptionId,
    pub mailbox_id: MailBoxId,
    pub name: Option<SmallStr>,
    pub event: SmallStr,
    pub stats: SubscriptionStats,
}
