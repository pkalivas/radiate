use crate::{MailBoxId, SubscriptionId};
use radiate_core::SmallStr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StreamSnapshot {
    pub event_count: usize,
    pub events: Vec<EventStats>,
    pub subscribers: Vec<SubscriptionSnapshot>,
}

impl StreamSnapshot {
    pub fn stream_metrics(&self) -> DeliveryMetrics {
        self.subscribers
            .iter()
            .fold(DeliveryMetrics::default(), |acc, sub| acc.add(&sub.stats))
    }

    pub fn mailbox_metrics(&self) -> HashMap<MailBoxId, DeliveryMetrics> {
        self.subscribers.iter().fold(
            HashMap::<MailBoxId, DeliveryMetrics>::new(),
            |mut acc, sub| {
                acc.entry(sub.mailbox_id)
                    .and_modify(|metrics| *metrics = metrics.add(&sub.stats))
                    .or_insert_with(|| sub.stats.clone());
                acc
            },
        )
    }
}

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DeliveryMetrics {
    /// Events routed to its mailbox.
    pub delivered: u64,
    /// Events its schedule said weren't due.
    pub skipped: u64,
    /// Deliveries its handler finished.
    pub handled: u64,
    /// Of `handled`, how many ran straight from the publisher's call rather
    /// than from the mailbox's queue. Their `wait` is only the time other
    /// subscribers to the same event took before them.
    pub inline: u64,
    /// Deliveries dropped unhandled: its subscription ended while they were
    /// queued, or an earlier panic in its mailbox discarded them.
    pub discarded: u64,
    /// Deliveries whose handler panicked.
    pub panicked: u64,
    /// Total time from publish until its handler started, across every
    /// handled delivery.
    pub wait: Duration,
    /// Total time spent inside its handler, across every handled delivery.
    pub handling: Duration,
}

impl DeliveryMetrics {
    pub fn queued(&self) -> u64 {
        self.delivered
            .saturating_sub(self.handled + self.discarded + self.panicked)
    }

    pub fn mean_wait(&self) -> Option<Duration> {
        self.mean(self.wait)
    }

    pub fn mean_handling(&self) -> Option<Duration> {
        self.mean(self.handling)
    }

    pub fn mean_total(&self) -> Option<Duration> {
        self.mean(self.wait + self.handling)
    }

    /// Deliveries per second its handlers could get through if they did
    /// nothing else: `handled / handling`. `None` until there's handling time
    /// to divide by, which a run of very fast handlers can leave at zero.
    pub fn capacity(&self) -> Option<f64> {
        let seconds = self.handling.as_secs_f64();
        (self.handled > 0 && seconds > 0.0).then(|| self.handled as f64 / seconds)
    }

    fn mean(&self, total: Duration) -> Option<Duration> {
        match self.handled {
            0 => None,
            n => Some(Duration::from_nanos(
                (total.as_nanos() / n as u128)
                    .try_into()
                    .unwrap_or(u64::MAX),
            )),
        }
    }

    pub fn add(mut self, other: &Self) -> Self {
        self.delivered += other.delivered;
        self.skipped += other.skipped;
        self.handled += other.handled;
        self.inline += other.inline;
        self.discarded += other.discarded;
        self.panicked += other.panicked;
        self.wait += other.wait;
        self.handling += other.handling;

        self
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SubscriptionSnapshot {
    pub id: SubscriptionId,
    pub mailbox_id: MailBoxId,
    pub name: Option<SmallStr>,
    pub event: SmallStr,
    pub stats: DeliveryMetrics,
}

impl SubscriptionSnapshot {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_account_for_every_delivery() {
        let stats = DeliveryMetrics {
            delivered: 10,
            handled: 5,
            discarded: 2,
            panicked: 1,
            ..Default::default()
        };
        assert_eq!(stats.queued(), 2);
    }
}
