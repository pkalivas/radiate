use crate::{
    EventContext, EventHandler, EventStats, Handler, LogEvent, LogLevel, StreamClosed,
    StreamSnapshot, SubscriptionId, SubscriptionInfo, Warning, events::messages::StreamEvent,
};
use radiate_core::{SmallStr, error::RadiateResult};
use std::collections::HashMap;
use std::time::Duration;

/// Snapshots in a row with a growing queue before a handler is reported as
/// falling behind.
const DEFAULT_FALLING_BEHIND_SNAPSHOTS: usize = 5;

/// Watches the stream's own [`StreamSnapshot`]s and reports how each handler
/// is keeping up and how often each event is published.
///
/// - Publishes a [`Warning`] when a handler's queue (delivered but not yet
///   handled) grows for several snapshots in a row. Each streak warns once.
/// - Logs a summary when the stream closes: per handler, events handled, the
///   rate over the run, peak queue length, and anything still queued; per
///   event, how many were published, never built, or delivered to nobody.
///
/// Snapshots are published once per generation, but only while something is
/// subscribed to them, so the stream pays nothing until this is attached. To
/// sample less often, put a schedule on its snapshot subscription.
///
/// Rates are measured between snapshots' own timestamps, so they stay right
/// even when this runs well behind the engine on a worker pool.
///
/// On a serial executor every handler runs inline, so queues are always
/// zero and only the totals and rates mean anything.
pub struct StreamMonitor {
    falling_behind_after: usize,
    handlers: HashMap<SubscriptionId, Tracked>,
    events: HashMap<SmallStr, TrackedEvent>,
    first: Option<Duration>,
    last: Option<Duration>,
    snapshots: usize,
}

struct Tracked {
    info: SubscriptionInfo,
    first_handled: u64,
    peak_queued: u64,
    growing: usize,
    warned: usize,
}

struct TrackedEvent {
    stats: EventStats,
    first_published: u64,
}

impl StreamMonitor {
    pub fn new() -> Self {
        StreamMonitor {
            falling_behind_after: DEFAULT_FALLING_BEHIND_SNAPSHOTS,
            handlers: HashMap::new(),
            events: HashMap::new(),
            first: None,
            last: None,
            snapshots: 0,
        }
    }

    /// How many snapshots in a row a queue has to grow before warning.
    pub fn falling_behind_after(mut self, snapshots: usize) -> Self {
        self.falling_behind_after = snapshots.max(1);
        self
    }

    fn elapsed(&self) -> f64 {
        match (self.first, self.last) {
            (Some(first), Some(last)) => (last - first).as_secs_f64(),
            _ => 0.0,
        }
    }

    fn summary(&self) -> String {
        let elapsed = self.elapsed();

        let mut handlers = self
            .handlers
            .values()
            .map(|tracked| {
                let stats = &tracked.info.stats;
                let (handler, event) = split_name(&tracked.info);
                let handled_since = stats.handled.saturating_sub(tracked.first_handled);

                vec![
                    handler,
                    event,
                    stats.handled.to_string(),
                    stats.skipped.to_string(),
                    rate(handled_since, elapsed),
                    tracked.peak_queued.to_string(),
                    stats.queued().to_string(),
                    match tracked.warned {
                        0 => String::new(),
                        n => format!("fell behind {n}x"),
                    },
                ]
            })
            .collect::<Vec<_>>();
        handlers.sort();

        // Show each handler's name once, on the first row of its group.
        for i in (1..handlers.len()).rev() {
            if handlers[i][0] == handlers[i - 1][0] {
                handlers[i][0].clear();
            }
        }

        let mut events = self
            .events
            .values()
            .map(|tracked| {
                let stats = &tracked.stats;
                let published_since = stats.published.saturating_sub(tracked.first_published);

                vec![
                    short_type_name(&stats.event),
                    stats.published.to_string(),
                    rate(published_since, elapsed),
                    stats.unbuilt.to_string(),
                    stats.undelivered.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        events.sort();

        let mut out = format!(
            "Stream | {} subscriptions | {} events | {} snapshots over {:.2}s",
            handlers.len(),
            events.len(),
            self.snapshots,
            elapsed
        );

        out.push_str(&table(&HANDLER_COLUMNS, &handlers));
        out.push('\n');
        out.push_str(&table(&EVENT_COLUMNS, &events));
        out
    }

    fn handle_snap(&mut self, snapshot: &StreamSnapshot, ctx: &EventContext<'_, Self>) {
        self.first.get_or_insert(snapshot.at);
        self.last = Some(snapshot.at);
        self.snapshots += 1;

        for stats in &snapshot.events {
            self.events
                .entry(stats.event.clone())
                .and_modify(|tracked| tracked.stats = stats.clone())
                .or_insert_with(|| TrackedEvent {
                    stats: stats.clone(),
                    first_published: stats.published,
                });
        }

        let own = ctx.mailbox_id();
        for info in snapshot.subscribers.iter().filter(|s| s.mailbox_id != own) {
            let queued = info.stats.queued();

            let tracked = self.handlers.entry(info.id).or_insert_with(|| Tracked {
                info: info.clone(),
                first_handled: info.stats.handled,
                peak_queued: 0,
                growing: 0,
                warned: 0,
            });

            let previous = tracked.info.stats.queued();
            tracked.growing = if queued > previous {
                tracked.growing + 1
            } else {
                0
            };

            if tracked.growing == self.falling_behind_after {
                tracked.warned += 1;
                ctx.publish(Warning(format!(
                    "Stream | {} is falling behind: {} events queued, growing for {} snapshots",
                    label(info),
                    queued,
                    tracked.growing
                )));
            }

            tracked.peak_queued = tracked.peak_queued.max(queued);
            tracked.info = info.clone();
        }
    }
}

#[derive(Clone, Copy)]
enum Align {
    Left,
    Right,
    /// Left-aligned and unpadded: only for the last column, so lines don't
    /// trail spaces.
    Notes,
}

const HANDLER_COLUMNS: [(&str, Align); 8] = [
    ("handler", Align::Left),
    ("event", Align::Left),
    ("handled", Align::Right),
    ("skipped", Align::Right),
    ("rate", Align::Right),
    ("peak", Align::Right),
    ("queued", Align::Right),
    ("", Align::Notes),
];

const EVENT_COLUMNS: [(&str, Align); 5] = [
    ("event", Align::Left),
    ("published", Align::Right),
    ("rate", Align::Right),
    ("unbuilt", Align::Right),
    ("undelivered", Align::Right),
];

/// Render `rows` under a header and a dash rule, each line indented and
/// starting with a newline.
fn table(columns: &[(&str, Align)], rows: &[Vec<String>]) -> String {
    let header = columns
        .iter()
        .map(|(name, _)| name.to_string())
        .collect::<Vec<_>>();
    let widths = (0..columns.len())
        .map(|col| {
            std::iter::once(&header)
                .chain(rows)
                .map(|row| row[col].len())
                .max()
                .unwrap_or(0)
        })
        .collect::<Vec<_>>();

    let padded = columns
        .iter()
        .zip(&widths)
        .filter(|((_, align), _)| !matches!(align, Align::Notes))
        .map(|(_, width)| width)
        .collect::<Vec<_>>();
    let rule = padded.iter().copied().sum::<usize>() + 2 * padded.len().saturating_sub(1);

    let mut out = String::new();
    for (i, row) in std::iter::once(&header).chain(rows).enumerate() {
        out.push_str("\n  ");
        for (cell, ((_, align), &width)) in row.iter().zip(columns.iter().zip(&widths)) {
            match align {
                Align::Left => out.push_str(&format!("{cell:<width$}  ")),
                Align::Right => out.push_str(&format!("{cell:>width$}  ")),
                Align::Notes => out.push_str(cell),
            }
        }
        out.truncate(out.trim_end().len());

        if i == 0 {
            out.push_str("\n  ");
            out.push_str(&"-".repeat(rule));
        }
    }

    out
}

/// Split a subscription name like `handler.engine.logger.event.epoch.complete`
/// into (`engine.logger`, `epoch.complete`). Plain subscriptions are named by
/// their event alone and get `-` as the handler; an unnamed one shows its
/// event's type name without the module path.
fn split_name(info: &SubscriptionInfo) -> (String, String) {
    let name = label(info);
    if let Some((handler, event)) = name.split_once(".event.") {
        let handler = handler.strip_prefix("handler.").unwrap_or(handler);
        (handler.to_string(), event.to_string())
    } else if let Some(event) = name.strip_prefix("event.") {
        ("-".to_string(), event.to_string())
    } else {
        ("-".to_string(), short_type_name(&name))
    }
}

/// `radiate::events::EpochComplete<alloc::vec::Vec<f32>>` -> `EpochComplete<Vec<f32>>`.
fn short_type_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut segment = String::new();
    for c in name.chars() {
        if c.is_alphanumeric() || c == '_' {
            segment.push(c);
        } else if c == ':' {
            segment.clear();
        } else {
            out.push_str(&segment);
            segment.clear();
            out.push(c);
        }
    }
    out.push_str(&segment);
    out
}

fn rate(handled: u64, elapsed: f64) -> String {
    if handled == 0 || elapsed <= 0.0 {
        return "-".to_string();
    }

    match handled as f64 / elapsed {
        r if r >= 1_000_000.0 => format!("{:.1}M/s", r / 1_000_000.0),
        r if r >= 1_000.0 => format!("{:.1}k/s", r / 1_000.0),
        r if r >= 10.0 => format!("{r:.0}/s"),
        r => format!("{r:.1}/s"),
    }
}

impl Default for StreamMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler for StreamMonitor {
    fn start(&mut self, ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
        ctx.subscribe::<StreamEvent<StreamSnapshot>>();
        ctx.subscribe::<StreamEvent<StreamClosed>>();
        Ok(())
    }
}

impl Handler<StreamEvent<StreamSnapshot>> for StreamMonitor {
    fn handle(&mut self, event: &StreamEvent<StreamSnapshot>, ctx: &EventContext<'_, Self>) {
        self.handle_snap(event, ctx);
    }
}

impl Handler<StreamEvent<StreamClosed>> for StreamMonitor {
    fn handle(&mut self, event: &StreamEvent<StreamClosed>, ctx: &EventContext<'_, Self>) {
        if self.snapshots > 0 {
            self.handle_snap(&event.0.0, ctx);
            ctx.publish(LogEvent(LogLevel::Info, self.summary()));
        }
    }
}

fn label(info: &SubscriptionInfo) -> String {
    match &info.name {
        Some(name) => name.to_string(),
        None => info.event.to_string(),
    }
}
