use crate::{
    EventContext, EventHandler, Handler, LogEvent, LogLevel, StreamClosed, StreamSnapshot,
    SubscriptionId, SubscriptionInfo, Warning, events::messages::StreamEvent,
};
use radiate_core::error::RadiateResult;
use std::collections::HashMap;
use std::time::Instant;

/// Snapshots in a row with a growing queue before a handler is reported as
/// falling behind.
const DEFAULT_FALLING_BEHIND_SNAPSHOTS: usize = 5;

/// Watches the stream's own [`StreamEvent::Stats`] snapshots and reports how
/// each handler is keeping up.
///
/// - Publishes a [`Warning`] when a handler's queue (delivered but not yet
///   handled) grows for several snapshots in a row. Each streak warns once.
/// - Logs a per-handler summary when the engine stops: events handled, the
///   rate over the run, peak queue length, and anything still queued.
///
/// Snapshots are published once per generation, but only while something is
/// subscribed to them, so the stream pays nothing until this is attached. To
/// sample less often, put a schedule on its `StreamEvent` subscription.
///
/// On a serial executor every handler runs inline, so queues are always
/// zero and only the totals and rates mean anything.
pub struct StreamMonitor {
    falling_behind_after: usize,
    handlers: HashMap<SubscriptionId, Tracked>,
    first: Option<Instant>,
    last: Option<Instant>,
    snapshots: usize,
}

struct Tracked {
    info: SubscriptionInfo,
    first_handled: u64,
    peak_queued: u64,
    growing: usize,
    warned: usize,
}

impl StreamMonitor {
    pub fn new() -> Self {
        StreamMonitor {
            falling_behind_after: DEFAULT_FALLING_BEHIND_SNAPSHOTS,
            handlers: HashMap::new(),
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

    fn summary(&self) -> String {
        let elapsed = match (self.first, self.last) {
            (Some(first), Some(last)) => (last - first).as_secs_f64(),
            _ => 0.0,
        };

        let mut rows = self
            .handlers
            .values()
            .map(|tracked| {
                let stats = &tracked.info.stats;
                let (handler, event) = split_name(&tracked.info);
                let handled_since = stats.handled.saturating_sub(tracked.first_handled);

                [
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
        rows.sort();

        // Show each handler's name once, on the first row of its group.
        for i in (1..rows.len()).rev() {
            if rows[i][0] == rows[i - 1][0] {
                rows[i][0].clear();
            }
        }

        let header = HEADER.map(String::from);
        let widths = (0..HEADER.len())
            .map(|col| {
                std::iter::once(&header)
                    .chain(&rows)
                    .map(|row| row[col].len())
                    .max()
                    .unwrap_or(0)
            })
            .collect::<Vec<_>>();

        let mut out = format!(
            "Stream | {} subscriptions | {} snapshots over {:.2}s",
            rows.len(),
            self.snapshots,
            elapsed
        );

        let rule = widths.iter().take(HEADER.len() - 1).sum::<usize>() + 2 * (HEADER.len() - 2);
        for (i, row) in std::iter::once(&header).chain(&rows).enumerate() {
            out.push_str("\n  ");
            for (col, cell) in row.iter().enumerate() {
                let width = widths[col];
                match col {
                    0 | 1 => out.push_str(&format!("{cell:<width$}  ")),
                    // last column (notes) isn't padded, so lines don't trail spaces
                    _ if col == HEADER.len() - 1 => out.push_str(cell),
                    _ => out.push_str(&format!("{cell:>width$}  ")),
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

    fn handle_snap(&mut self, event: &StreamSnapshot, ctx: &EventContext<'_, Self>) {
        let stats = &event;

        let now = Instant::now();
        self.first.get_or_insert(now);
        self.last = Some(now);
        self.snapshots += 1;

        let own = ctx.mailbox_id();
        for info in stats.subscribers.iter().filter(|s| s.mailbox_id != own) {
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

const HEADER: [&str; 8] = [
    "handler", "event", "handled", "skipped", "rate", "peak", "queued", "",
];

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
