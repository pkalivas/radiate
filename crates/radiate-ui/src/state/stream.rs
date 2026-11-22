use radiate_engines::{DeliveryMetrics, StreamSnapshot, SubscriptionId, SubscriptionSnapshot};
use std::{collections::HashMap, time::Duration};

/// How much of the stream's open time each window covers. Snapshots arrive
/// every few generations, so measuring between consecutive ones would jitter;
/// instead the tables show the last full window, refreshed once per window.
const WINDOW: Duration = Duration::from_secs(1);

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
    /// Publishes per second of the stream's open time, over the last window.
    pub rate: Option<f64>,
    /// Handler time one publish of this event costs: the mean handling time
    /// of every subscriber to it, summed. `None` until one has handled it.
    pub cost: Option<Duration>,
    pub unbuilt: u64,
    pub undelivered: u64,
}

/// The latest [`StreamSnapshot`], turned into rows for the stream tables.
///
/// Counts are totals. Rates and times are over the last full window, so they
/// follow what the stream is doing now; a handler with nothing in that window
/// falls back to its averages since it subscribed.
#[derive(Default)]
pub struct StreamState {
    pub handlers: Vec<HandlerRow>,
    pub events: Vec<EventRow>,
    /// Handled deliveries per second of open time, over the last window.
    pub throughput: Option<f64>,
    /// Share of open time spent in handlers over the last window. Can exceed
    /// 1.0 on a worker pool, where handlers run side by side.
    pub utilization: Option<f64>,
    /// The snapshot the current window started at.
    window_start: Option<StreamSnapshot>,
    /// What changed over the last full window.
    window: Option<StreamSnapshot>,
    /// Queue depths from the previous snapshot, to tell whether each is growing.
    queued: HashMap<SubscriptionId, u64>,
}

impl StreamState {
    pub fn update(&mut self, snapshot: StreamSnapshot) {
        self.roll_window(&snapshot);

        let recent = self
            .window
            .iter()
            .flat_map(|window| &window.subscribers)
            .map(|info| (info.id, info.stats))
            .collect::<HashMap<_, _>>();
        // The last window's stats if it handled anything, else the totals.
        let stats_for = |info: &SubscriptionSnapshot| -> DeliveryMetrics {
            recent
                .get(&info.id)
                .filter(|stats| stats.handled > 0)
                .copied()
                .unwrap_or(info.stats)
        };

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
                let stats = stats_for(info);
                HandlerRow {
                    handler,
                    event,
                    handled: info.stats.handled,
                    capacity: stats.capacity(),
                    time: stats.mean_handling(),
                    wait: stats.mean_wait(),
                    queued,
                    growing: queued > previous.get(&info.id).copied().unwrap_or(0),
                }
            })
            .collect();
        self.handlers
            .sort_by(|a, b| (&a.handler, &a.event).cmp(&(&b.handler, &b.event)));

        let window = self.window.as_ref();
        self.events = snapshot
            .events
            .iter()
            .filter(|stats| !is_internal(&stats.event))
            .map(|stats| EventRow {
                event: stats.short_name(),
                published: stats.published,
                rate: window.and_then(|w| {
                    let recent = w.events.iter().find(|e| e.event == stats.event)?;
                    let seconds = w.elapsed.as_secs_f64();
                    (seconds > 0.0).then(|| recent.published as f64 / seconds)
                }),
                cost: subscribers
                    .iter()
                    .filter(|info| info.event == stats.event)
                    .filter_map(|info| stats_for(info).mean_handling())
                    .reduce(|a, b| a + b),
                unbuilt: stats.unbuilt,
                undelivered: stats.undelivered,
            })
            .collect();
        self.events.sort_by(|a, b| a.event.cmp(&b.event));

        self.throughput = window.and_then(StreamSnapshot::throughput);
        self.utilization = window.and_then(StreamSnapshot::utilization);
    }

    /// Once a full window of open time has passed, keep what changed over it
    /// and start the next one at `snapshot`.
    fn roll_window(&mut self, snapshot: &StreamSnapshot) {
        let Some(start) = &self.window_start else {
            self.window_start = Some(snapshot.clone());
            return;
        };

        if snapshot.elapsed.saturating_sub(start.elapsed) < WINDOW {
            return;
        }

        self.window = Some(snapshot.since(start));
        self.window_start = Some(snapshot.clone());
    }
}

/// The stream's own bookkeeping events (snapshots, subscription changes),
/// which would only clutter the tables.
fn is_internal(event: &str) -> bool {
    event.contains("StreamEvent<")
}
