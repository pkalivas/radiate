use crate::{SubscriptionStats, events::mailbox::MailBoxId};
use radiate_core::{Expr, RadiateError, error::RadiateResult, radiate_err};
use radiate_error::radiate_bail;
use radiate_utils::{SmallStr, sentry_id};
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::{
    fmt::Debug,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
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
}

#[derive(Clone)]
pub struct Subscription {
    pub(crate) id: SubscriptionId,
    pub(crate) mailbox_id: MailBoxId,
    pub(crate) name: Arc<RwLock<Option<SmallStr>>>,
    pub(crate) schedule: Arc<RwLock<Option<Expr>>>,
    pub(crate) permits: Arc<AtomicUsize>,
    pub(crate) alive: Arc<AtomicBool>,
    pub(crate) is_scheduled: Arc<AtomicBool>,
    pub(crate) counters: Arc<Counters>,
}

impl Subscription {
    pub(super) fn new(mailbox_id: MailBoxId) -> Self {
        Subscription {
            id: SubscriptionId::new(),
            mailbox_id,
            name: Arc::new(RwLock::new(None)),
            schedule: Arc::new(RwLock::new(None)),
            permits: Arc::new(AtomicUsize::new(0)),
            alive: Arc::new(AtomicBool::new(true)),
            is_scheduled: Arc::new(AtomicBool::new(false)),
            counters: Arc::new(Counters::default()),
        }
    }

    /// Attach a name to this subscription, used to identify it in
    /// [`EventStream::stats`](crate::events::EventStream::stats).
    pub fn named(self, name: impl Into<SmallStr>) -> Self {
        *self.name.write().unwrap() = Some(name.into());
        self
    }

    pub fn name(&self) -> Option<SmallStr> {
        self.name.read().unwrap().clone()
    }

    /// A snapshot of how many events this subscription has been delivered,
    /// skipped, and handled.
    pub fn stats(&self) -> SubscriptionStats {
        SubscriptionStats {
            delivered: self.counters.delivered.load(Ordering::Relaxed),
            skipped: self.counters.skipped.load(Ordering::Relaxed),
            handled: self.counters.handled.load(Ordering::Relaxed),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }

    pub fn id(&self) -> SubscriptionId {
        self.id
    }

    pub fn mailbox_id(&self) -> MailBoxId {
        self.mailbox_id
    }

    pub fn schedule(&self, schedule: impl Into<Expr>) -> Result<bool, RadiateError> {
        let maybe_expr = schedule.into();

        match maybe_expr.clone().into_schedule() {
            Some(expr) => {
                *self.schedule.write().unwrap() = Some(expr);
                self.is_scheduled.store(true, Ordering::Release);
                Ok(true)
            }
            _ => {
                radiate_bail!(Expr: format!("Invalid schedule expression: {:?}", maybe_expr))
            }
        }
    }

    pub fn unsubscribe(&self) {
        self.alive.store(false, Ordering::Release);
        self.permits.store(0, Ordering::Release);
    }

    pub(super) fn reserve(&self) -> RadiateResult<bool> {
        if !self.is_alive() {
            return Ok(false);
        }

        if !self.try_schedule()? {
            self.counters.record_skipped();
            return Ok(false);
        }

        self.permits.fetch_add(1, Ordering::Release);

        Ok(true)
    }

    /// For eager publishes: advance the schedule and say whether this event is due.
    pub(super) fn is_due(&self) -> bool {
        if !self.is_alive() {
            return false;
        }

        // Same as lazy_publish: an expression that fails to evaluate counts as not due.
        let due = self.try_schedule().unwrap_or(false);
        if !due {
            self.counters.record_skipped();
        }
        due
    }

    pub(super) fn take_permit(&self) -> bool {
        let mut current = self.permits.load(Ordering::Acquire);

        loop {
            if current == 0 {
                return false;
            }

            match self.permits.compare_exchange_weak(
                current,
                current - 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return true,
                Err(next) => current = next,
            }
        }
    }

    fn try_schedule(&self) -> RadiateResult<bool> {
        if !self.is_scheduled.load(Ordering::Acquire) {
            return Ok(true); // fast path: no lock, no expression
        }

        let mut guard = self.schedule.write().unwrap();
        if let Some(expr) = &mut *guard {
            return expr
                .trigger()?
                .extract_bool()
                .ok_or_else(|| radiate_err!(Expr: "Failed to compute schedule as bool"));
        }

        Ok(true)
    }
}

impl Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subscription(id={})", self.id)
    }
}
