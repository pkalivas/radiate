use radiate_engines::{StreamSnapshot, SubscriptionId, SubscriptionSnapshot};
use std::{collections::HashMap, time::Duration};

pub struct HandlerRow {
    pub handler: String,
    pub event: String,
    pub handled: u64,
    /// Events per second the handler could keep up with: one over its mean
    /// handling time. `None` until it has handled something.
    pub capacity: Option<f64>,
    /// Mean time inside the handler.
    pub time: Option<Duration>,
    /// Mean time from publish until the handler started.
    pub wait: Option<Duration>,
    pub queued: u64,
    /// The queue grew since the previous snapshot.
    pub growing: bool,
}

pub struct EventRow {
    pub event: String,
    pub published: u64,
    /// Handler time one publish of this event costs: the mean handling time
    /// of every subscriber to it, summed. `None` until one has handled it.
    pub cost: Option<Duration>,
    pub unbuilt: u64,
    pub undelivered: u64,
}

/// The latest [`StreamSnapshot`], turned into rows for the stream tables.
/// Times are averages since each subscription started.
#[derive(Default)]
pub struct StreamState {
    pub handlers: Vec<HandlerRow>,
    pub events: Vec<EventRow>,
    /// Queue depths from the previous snapshot, to tell whether each is growing.
    queued: HashMap<SubscriptionId, u64>,
}

impl StreamState {
    pub fn update(&mut self, snapshot: StreamSnapshot) {
        let subscribers = snapshot
            .subscribers
            .iter()
            .filter(|info| !is_internal(&info.event))
            .collect::<Vec<_>>();

        let previous = std::mem::take(&mut self.queued);

        self.handlers = subscribers
            .iter()
            .map(|info| {
                let (handler, event) = info.labels();
                let queued = info.stats.queued();
                self.queued.insert(info.id, queued);
                HandlerRow {
                    handler,
                    event,
                    handled: info.stats.handled,
                    capacity: info.stats.capacity(),
                    time: info.stats.mean_handling(),
                    wait: info.stats.mean_wait(),
                    queued,
                    growing: queued > previous.get(&info.id).copied().unwrap_or(0),
                }
            })
            .collect();
        self.handlers
            .sort_by(|a, b| (&a.handler, &a.event).cmp(&(&b.handler, &b.event)));

        self.events = snapshot
            .events
            .iter()
            .filter(|stats| !is_internal(&stats.event))
            .map(|stats| EventRow {
                event: stats.short_name(),
                published: stats.published,
                cost: cost_per_publish(subscribers.iter().filter(|info| info.event == stats.event)),
                unbuilt: stats.unbuilt,
                undelivered: stats.undelivered,
            })
            .collect();
        self.events.sort_by(|a, b| a.event.cmp(&b.event));
    }
}

/// The summed mean handling time of `subscribers`, or `None` if none of them
/// has handled anything yet.
fn cost_per_publish<'a>(
    subscribers: impl Iterator<Item = &'a &'a SubscriptionSnapshot>,
) -> Option<Duration> {
    subscribers
        .filter_map(|info| info.stats.mean_handling())
        .reduce(|a, b| a + b)
}

/// The stream's own bookkeeping events (snapshots, subscription changes),
/// which would only clutter the tables.
fn is_internal(event: &str) -> bool {
    event.contains("StreamEvent<")
}
