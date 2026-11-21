use radiate_engines::{StreamSnapshot, SubscriptionId};
use radiate_utils::SmallStr;
use std::{collections::HashMap, time::Duration};

/// How long a window rates are measured over. Snapshots arrive every
/// generation, so rates between consecutive ones would jitter; instead they
/// are taken over the last full window and refreshed once per window.
const RATE_WINDOW: Duration = Duration::from_secs(1);

pub struct HandlerRow {
    pub handler: String,
    pub event: String,
    pub handled: u64,
    pub rate: Option<f64>,
    pub queued: u64,
    /// The queue grew since the previous snapshot.
    pub growing: bool,
}

pub struct EventRow {
    pub event: String,
    pub published: u64,
    pub rate: Option<f64>,
    pub unbuilt: u64,
    pub undelivered: u64,
}

/// The latest [`StreamSnapshot`], turned into rows for the stream tables.
#[derive(Default)]
pub struct StreamState {
    pub handlers: Vec<HandlerRow>,
    pub events: Vec<EventRow>,
    window: Option<StreamSnapshot>,
    handler_rates: HashMap<SubscriptionId, f64>,
    event_rates: HashMap<SmallStr, f64>,
    queued: HashMap<SubscriptionId, u64>,
}

impl StreamState {
    pub fn update(&mut self, snapshot: StreamSnapshot) {
        self.roll_window(&snapshot);

        self.handlers = snapshot
            .subscribers
            .iter()
            .filter(|info| !is_internal(&info.event))
            .map(|info| {
                let (handler, event) = info.labels();
                let queued = info.stats.queued();
                let previous = self.queued.insert(info.id, queued).unwrap_or(0);
                HandlerRow {
                    handler,
                    event,
                    handled: info.stats.handled,
                    rate: self.handler_rates.get(&info.id).copied(),
                    queued,
                    growing: queued > previous,
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
                rate: self.event_rates.get(&stats.event).copied(),
                unbuilt: stats.unbuilt,
                undelivered: stats.undelivered,
            })
            .collect();
        self.events.sort_by(|a, b| a.event.cmp(&b.event));
    }

    /// Once a full window has passed, measure rates over it and start the
    /// next one at `snapshot`.
    fn roll_window(&mut self, snapshot: &StreamSnapshot) {
        let Some(start) = &self.window else {
            self.window = Some(snapshot.clone());
            return;
        };

        let elapsed = snapshot.at.saturating_sub(start.at);
        if elapsed < RATE_WINDOW {
            return;
        }

        let secs = elapsed.as_secs_f64();
        let handled_before = start
            .subscribers
            .iter()
            .map(|info| (info.id, info.stats.handled))
            .collect::<HashMap<_, _>>();
        let published_before = start
            .events
            .iter()
            .map(|stats| (&stats.event, stats.published))
            .collect::<HashMap<_, _>>();

        self.handler_rates = snapshot
            .subscribers
            .iter()
            .map(|info| {
                (
                    info.id,
                    since(&handled_before, &info.id, info.stats.handled, secs),
                )
            })
            .collect();
        self.event_rates = snapshot
            .events
            .iter()
            .map(|stats| {
                let rate = since(&published_before, &&stats.event, stats.published, secs);
                (stats.event.clone(), rate)
            })
            .collect();

        self.window = Some(snapshot.clone());
    }
}

fn since<K: Eq + std::hash::Hash>(before: &HashMap<K, u64>, key: &K, now: u64, secs: f64) -> f64 {
    let before = before.get(key).copied().unwrap_or(0);
    now.saturating_sub(before) as f64 / secs
}

/// The stream's own bookkeeping events (snapshots, subscription changes),
/// which would only clutter the tables.
fn is_internal(event: &str) -> bool {
    event.contains("StreamEvent<")
}
