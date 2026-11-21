use crate::{SubscriptionStats, events::mailbox::MailBoxId};
use radiate_core::{Expr, error::RadiateResult};
use radiate_error::radiate_bail;
use radiate_utils::{SmallStr, sentry_id};
use std::{
    fmt::Debug,
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

sentry_id!(SubscriptionId);

/// Live counters shared between a [`Subscription`] handle, the stream that
/// dispatches to it, and the subscriber that handles its events.
#[derive(Default)]
pub(crate) struct Counters {
    delivered: AtomicU64,
    skipped: AtomicU64,
    handled: AtomicU64,
}

impl Counters {
    #[inline]
    pub(crate) fn record_delivered(&self) {
        self.delivered.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn record_skipped(&self) {
        self.skipped.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    pub(crate) fn record_handled(&self) {
        self.handled.fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot(&self) -> SubscriptionStats {
        SubscriptionStats {
            delivered: self.delivered.load(Ordering::Relaxed),
            skipped: self.skipped.load(Ordering::Relaxed),
            handled: self.handled.load(Ordering::Relaxed),
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

    /// A snapshot of how many events this subscription has been delivered,
    /// skipped, and handled.
    pub fn stats(&self) -> SubscriptionStats {
        self.0.counters.snapshot()
    }

    #[inline]
    pub fn is_alive(&self) -> bool {
        self.0.alive.load(Ordering::Acquire)
    }

    /// Only deliver events when `schedule` evaluates to true. It's evaluated
    /// once per published event.
    pub fn schedule(&self, schedule: impl Into<Expr>) -> RadiateResult<()> {
        let expr = schedule.into();
        let Some(schedule) = expr.clone().into_schedule() else {
            radiate_bail!(Expr: format!("Invalid schedule expression: {:?}", expr));
        };

        *self.0.schedule.lock().unwrap() = Some(schedule);
        self.0.is_scheduled.store(true, Ordering::Release);
        Ok(())
    }

    /// Stop delivering to this subscription. Takes effect for any publish
    /// that hasn't reached it yet, including one already in progress.
    pub fn unsubscribe(&self) {
        self.0.alive.store(false, Ordering::Release);
    }

    #[inline]
    pub(super) fn counters(&self) -> &Arc<Counters> {
        &self.0.counters
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
            self.0.counters.record_skipped();
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

impl Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subscription(id={})", self.0.id)
    }
}
