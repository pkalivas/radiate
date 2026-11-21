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
    pub published: u64,
    pub unbuilt: u64,
    pub undelivered: u64,
}

impl EventStats {
    pub fn short_name(&self) -> String {
        radiate_utils::type_name_from_str(&self.event)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SubscriptionStats {
    pub delivered: u64,
    pub skipped: u64,
    pub handled: u64,
}

impl SubscriptionStats {
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

impl SubscriptionInfo {
    pub fn label(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.event)
    }

    pub fn labels(&self) -> (String, String) {
        let name = self.label();
        if let Some((handler, event)) = name.split_once(".event.") {
            let handler = handler.strip_prefix("handler.").unwrap_or(handler);
            (handler.to_string(), event.to_string())
        } else if let Some(event) = name.strip_prefix("event.") {
            ("-".to_string(), event.to_string())
        } else {
            ("-".to_string(), radiate_utils::type_name_from_str(name))
        }
    }
}
