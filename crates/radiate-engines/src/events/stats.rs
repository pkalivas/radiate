use crate::{MailBoxId, SubscriptionId};
use radiate_core::SmallStr;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StreamSnapshot {
    pub elapsed: Duration,
    pub event_count: usize,
    pub events: Vec<EventStats>,
    pub subscribers: Vec<SubscriptionSnapshot>,
}

impl StreamSnapshot {
    /// Deliveries handled per second the stream was open. On one snapshot,
    /// the average over its whole life; on one from [`since`](Self::since),
    /// over that interval. `None` before any open time has passed.
    pub fn throughput(&self) -> Option<f64> {
        self.per_second(self.stream_metrics().handled as f64)
    }

    /// Share of open time spent inside handlers: 0.02 means 2%. Handlers on
    /// a worker pool run side by side, so this can exceed 1.0; read it then
    /// as cores' worth of handler work.
    pub fn utilization(&self) -> Option<f64> {
        self.per_second(self.stream_metrics().handling.as_secs_f64())
    }

    /// What happened between `earlier` and `self`. Counts and times become the
    /// change over the interval, and `elapsed` becomes its length, so
    /// [`throughput`](Self::throughput) and [`utilization`](Self::utilization)
    /// give the rates over just that window. Event types and subscriptions
    /// that are new since `earlier` keep their full totals; ones that have
    /// gone are left out.
    pub fn since(&self, earlier: &StreamSnapshot) -> StreamSnapshot {
        let events = self
            .events
            .iter()
            .map(
                |stats| match earlier.events.iter().find(|e| e.event == stats.event) {
                    Some(before) => stats.since(before),
                    None => stats.clone(),
                },
            )
            .collect::<Vec<_>>();

        let subscribers = self
            .subscribers
            .iter()
            .map(
                |sub| match earlier.subscribers.iter().find(|s| s.id == sub.id) {
                    Some(before) => SubscriptionSnapshot {
                        stats: sub.stats.since(&before.stats),
                        ..sub.clone()
                    },
                    None => sub.clone(),
                },
            )
            .collect();

        StreamSnapshot {
            elapsed: self.elapsed.saturating_sub(earlier.elapsed),
            event_count: events.iter().map(|e| e.published as usize).sum(),
            events,
            subscribers,
        }
    }

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
                    .or_insert_with(|| sub.stats);
                acc
            },
        )
    }

    fn per_second(&self, amount: f64) -> Option<f64> {
        let seconds = self.elapsed.as_secs_f64();
        (seconds > 0.0).then(|| amount / seconds)
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

    /// What changed between `earlier` and `self`.
    pub fn since(&self, earlier: &Self) -> Self {
        EventStats {
            event: self.event.clone(),
            published: self.published.saturating_sub(earlier.published),
            unbuilt: self.unbuilt.saturating_sub(earlier.unbuilt),
            undelivered: self.undelivered.saturating_sub(earlier.undelivered),
        }
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

    /// What changed between `earlier` and `self`.
    pub fn since(&self, earlier: &Self) -> Self {
        DeliveryMetrics {
            delivered: self.delivered.saturating_sub(earlier.delivered),
            skipped: self.skipped.saturating_sub(earlier.skipped),
            handled: self.handled.saturating_sub(earlier.handled),
            inline: self.inline.saturating_sub(earlier.inline),
            discarded: self.discarded.saturating_sub(earlier.discarded),
            panicked: self.panicked.saturating_sub(earlier.panicked),
            wait: self.wait.saturating_sub(earlier.wait),
            handling: self.handling.saturating_sub(earlier.handling),
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

    fn snapshot(elapsed_ms: u64, handled: u64, handling_ms: u64) -> StreamSnapshot {
        StreamSnapshot {
            elapsed: Duration::from_millis(elapsed_ms),
            event_count: 0,
            events: vec![],
            subscribers: vec![SubscriptionSnapshot {
                id: SubscriptionId::new(),
                mailbox_id: MailBoxId::new(),
                name: None,
                event: "Ping".into(),
                stats: DeliveryMetrics {
                    delivered: handled,
                    handled,
                    handling: Duration::from_millis(handling_ms),
                    ..Default::default()
                },
            }],
        }
    }

    #[test]
    fn rates_are_per_second_of_open_time() {
        let snap = snapshot(2_000, 1_000, 500);
        assert_eq!(snap.throughput(), Some(500.0));
        assert_eq!(snap.utilization(), Some(0.25));
        assert_eq!(snapshot(0, 10, 1).throughput(), None, "never opened");
    }

    #[test]
    fn since_gives_rates_over_just_the_interval() {
        let earlier = snapshot(1_000, 100, 10);
        let mut later = snapshot(3_000, 1_100, 210);
        later.subscribers[0].id = earlier.subscribers[0].id;

        let window = later.since(&earlier);
        assert_eq!(window.elapsed, Duration::from_secs(2));
        assert_eq!(window.stream_metrics().handled, 1_000);
        assert_eq!(window.throughput(), Some(500.0));
        assert_eq!(window.utilization(), Some(0.1));
    }

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
