use crate::{DeliveryMetrics, events::mailbox::MailBoxId};
use crossbeam::utils::CachePadded;
use radiate_core::{Expr, error::RadiateResult};
use radiate_error::radiate_bail;
use radiate_utils::{SmallStr, sentry_id};
use std::{
    fmt::Debug,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

sentry_id!(SubscriptionId);

/// Live counters for one subscription. Two threads write them, so each
/// side gets its own cache line: on a pool, the publisher bumps `publish` on
/// every publish while a worker bumps `mailbox` on every handled event, and
/// sharing a line would bounce it between their cores on every event.
#[derive(Default)]
struct Counters {
    publish: CachePadded<PublishCounts>,
    mailbox: CachePadded<MailboxCounts>,
}

/// Written by the publishing thread.
#[derive(Default)]
struct PublishCounts {
    delivered: AtomicU64,
    skipped: AtomicU64,
}

/// Written by whichever thread is handling the mailbox.
#[derive(Default)]
struct MailboxCounts {
    handled: AtomicU64,
    inline: AtomicU64,
    discarded: AtomicU64,
    panicked: AtomicU64,
    wait_ns: AtomicU64,
    handling_ns: AtomicU64,
}

impl Counters {
    fn snapshot(&self) -> DeliveryMetrics {
        let (publish, mailbox) = (&self.publish, &self.mailbox);
        DeliveryMetrics {
            delivered: publish.delivered.load(Ordering::Relaxed),
            skipped: publish.skipped.load(Ordering::Relaxed),
            handled: mailbox.handled.load(Ordering::Relaxed),
            inline: mailbox.inline.load(Ordering::Relaxed),
            discarded: mailbox.discarded.load(Ordering::Relaxed),
            panicked: mailbox.panicked.load(Ordering::Relaxed),
            wait: Duration::from_nanos(mailbox.wait_ns.load(Ordering::Relaxed)),
            handling: Duration::from_nanos(mailbox.handling_ns.load(Ordering::Relaxed)),
        }
    }
}

struct Inner {
    id: SubscriptionId,
    mailbox_id: MailBoxId,
    name: RwLock<Option<SmallStr>>,
    schedule: Mutex<Option<Expr>>,
    /// Set once a schedule is installed, so unscheduled subscriptions never
    /// touch the lock.
    is_scheduled: AtomicBool,
    alive: AtomicBool,
    /// Its own allocation on purpose: the mailbox's thread writes these on
    /// every handled event, and inline they'd share a cache line with the
    /// flags the publisher reads on every publish (about 10% slower on a pool).
    counters: Arc<Counters>,
}

/// A handle to one handler's subscription to one event type. Clones share
/// the same subscription.
#[derive(Clone)]
pub struct Subscription(Arc<Inner>);

impl Subscription {
    pub(super) fn new(mailbox_id: MailBoxId) -> Self {
        Subscription(Arc::new(Inner {
            id: SubscriptionId::new(),
            mailbox_id,
            name: RwLock::new(None),
            schedule: Mutex::new(None),
            is_scheduled: AtomicBool::new(false),
            alive: AtomicBool::new(true),
            counters: Arc::default(),
        }))
    }

    pub fn id(&self) -> SubscriptionId {
        self.0.id
    }

    pub fn mailbox_id(&self) -> MailBoxId {
        self.0.mailbox_id
    }

    /// Attach a name to this subscription, used to identify it in
    /// [`EventStream::stats`](crate::events::EventStream::stats).
    pub fn with_name(self, name: impl Into<SmallStr>) -> Self {
        *self.0.name.write().unwrap() = Some(name.into());
        self
    }

    pub fn name(&self) -> Option<SmallStr> {
        self.0.name.read().unwrap().clone()
    }

    /// Everything this subscription has been sent so far and what became of
    /// it, including how long its events waited and how long its handler took.
    pub fn stats(&self) -> DeliveryMetrics {
        self.0.counters.snapshot()
    }

    #[inline]
    pub fn is_alive(&self) -> bool {
        self.0.alive.load(Ordering::Acquire)
    }

    /// Only deliver events when `schedule` evaluates to true. It's evaluated
    /// once per published event.
    pub fn schedule(&self, schedule: impl Into<Expr>) -> RadiateResult<Self> {
        let expr = schedule.into();
        let Some(schedule) = expr.clone().into_schedule() else {
            radiate_bail!(Expr: format!("Invalid schedule expression: {:?}", expr));
        };

        *self.0.schedule.lock().unwrap() = Some(schedule);
        self.0.is_scheduled.store(true, Ordering::Release);
        Ok(self.clone())
    }

    /// Stop delivering to this subscription. Takes effect for any publish
    /// that hasn't reached it yet, including one already in progress, and
    /// for deliveries already queued in its mailbox. A handler call already
    /// running finishes.
    pub fn unsubscribe(&self) {
        self.0.alive.store(false, Ordering::Release);
    }

    #[inline]
    pub(super) fn record_delivered(&self) {
        self.0
            .counters
            .publish
            .delivered
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Its handler finished a delivery published at `published_at`, having
    /// started on it at `started`.
    #[inline]
    pub(super) fn record_handled(
        &self,
        published_at: quanta::Instant,
        started: quanta::Instant,
        ended: quanta::Instant,
        inline: bool,
    ) {
        let counters = &self.0.counters.mailbox;
        counters.handled.fetch_add(1, Ordering::Relaxed);
        if inline {
            counters.inline.fetch_add(1, Ordering::Relaxed);
        }
        let wait = started.saturating_duration_since(published_at);
        counters.wait_ns.fetch_add(nanos(wait), Ordering::Relaxed);
        let handling = ended.saturating_duration_since(started);
        counters
            .handling_ns
            .fetch_add(nanos(handling), Ordering::Relaxed);
    }

    pub(super) fn record_panicked(&self) {
        self.0
            .counters
            .mailbox
            .panicked
            .fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn record_discarded(&self) {
        self.0
            .counters
            .mailbox
            .discarded
            .fetch_add(1, Ordering::Relaxed);
    }

    /// Advance the schedule and say whether this event is due. Called exactly
    /// once per live registration per published event, eager or lazy.
    #[inline]
    pub(super) fn tick(&self) -> bool {
        if !self.is_alive() {
            return false;
        }

        let due = self.try_schedule();
        if !due {
            self.0
                .counters
                .publish
                .skipped
                .fetch_add(1, Ordering::Relaxed);
        }
        due
    }

    #[inline]
    fn try_schedule(&self) -> bool {
        // Fast path: no lock, no expression.
        !self.0.is_scheduled.load(Ordering::Acquire) || self.evaluate_schedule()
    }

    /// An expression that fails to evaluate counts as not due.
    #[cold]
    fn evaluate_schedule(&self) -> bool {
        let mut schedule = self.0.schedule.lock().unwrap();
        let Some(expr) = schedule.as_mut() else {
            return true;
        };

        let due = expr.trigger().map(|value| value.extract_bool());
        matches!(due, Ok(Some(true)))
    }
}

#[inline]
fn nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

impl Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subscription(id={})", self.0.id)
    }
}
