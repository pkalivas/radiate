use radiate_engines::StreamSnapshot;
use std::{collections::HashMap, time::Duration};

pub struct HandlerRow {
    pub handler: String,
    pub event: String,
    pub handled: u64,
    pub rate: Option<f64>,
    pub time: Option<Duration>,
    pub wait: Option<Duration>,
    pub queued: u64,
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
    last_snap: Option<StreamSnapshot>,
}

impl StreamState {
    pub fn update(&mut self, snapshot: StreamSnapshot) {
        self.last_snap = Some(snapshot.clone());
        let mut subs_by_event = HashMap::new();

        self.handlers = snapshot
            .subscribers
            .iter()
            .filter(|info| !is_internal(&info.event))
            .map(|info| {
                let (handler, event) = info.labels();
                let queued = info.stats.queued();
                subs_by_event
                    .entry(info.event.clone())
                    .or_insert_with(Vec::new)
                    .push(info);

                HandlerRow {
                    handler,
                    event,
                    handled: info.stats.handled,
                    rate: info
                        .stats
                        .mean_total()
                        .map(|d| info.stats.handled as f64 / d.as_secs_f64()),
                    time: info.stats.mean_handling(),
                    wait: info.stats.mean_wait(),
                    queued,
                    growing: true,
                }
            })
            .collect();
        self.handlers
            .sort_by(|a, b| (&a.handler, &a.event).cmp(&(&b.handler, &b.event)));

        self.events = snapshot
            .events
            .iter()
            .filter(|stats| !is_internal(&stats.event))
            .map(|stats| {
                let current_sub = subs_by_event.get(stats.event.as_str());
                let mut total_handled = 0.0;
                let mut total_time = 0.0;
                if let Some(subs) = current_sub {
                    for info in subs {
                        if let Some(d) = info.stats.mean_total() {
                            total_handled += info.stats.handled as f64;
                            total_time += d.as_secs_f64();
                        }
                    }
                }

                let rate = if total_time > 0.0 {
                    Some(total_handled / total_time)
                } else {
                    None
                };
                EventRow {
                    event: stats.short_name(),
                    published: stats.published,
                    rate: rate,
                    unbuilt: stats.unbuilt,
                    undelivered: stats.undelivered,
                }
            })
            .collect();
        self.events.sort_by(|a, b| a.event.cmp(&b.event));
    }
}

/// The stream's own bookkeeping events (snapshots, subscription changes),
/// which would only clutter the tables.
fn is_internal(event: &str) -> bool {
    event.contains("StreamEvent<")
}
