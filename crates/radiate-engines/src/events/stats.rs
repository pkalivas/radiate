use crate::{MailBoxId, SubscriptionId};
use radiate_core::SmallStr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StreamStats {
    pub event_count: usize,
    pub subscribers: Vec<SubscriptionInfo>,
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
