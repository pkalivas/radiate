//! `Subscription`/`Schedule`/`SubscriptionId` — ported close to verbatim from
//! `radiate-engines/src/events/subscription.rs`. Throttling is orthogonal to the
//! `Message`/`Event` unification, so this carries over essentially unchanged: a subscription
//! can be scheduled independent of how often the publisher actually publishes.

use crate::sentry_id;
use std::sync::atomic::AtomicUsize;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, Ordering},
};

sentry_id!(SubscriptionId);

#[derive(Clone, Default)]
pub(super) enum Schedule {
    #[default]
    Always,
    EveryN(usize, Arc<AtomicUsize>),
    Duration(std::time::Duration, Arc<RwLock<std::time::Instant>>),
}

impl Schedule {
    fn is_scheduled(&self) -> bool {
        match self {
            Schedule::Always => true,
            Schedule::EveryN(n, counter) => {
                let current = counter.fetch_add(1, Ordering::Relaxed);
                current.saturating_add(1).is_multiple_of(*n)
            }
            Schedule::Duration(duration, last_time) => {
                let now = std::time::Instant::now();
                if now.duration_since(*last_time.read().unwrap()) >= *duration {
                    *last_time.write().unwrap() = now;
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl From<usize> for Schedule {
    fn from(n: usize) -> Self {
        Schedule::EveryN(n, Arc::new(AtomicUsize::new(0)))
    }
}

impl From<std::time::Duration> for Schedule {
    fn from(duration: std::time::Duration) -> Self {
        Schedule::Duration(duration, Arc::new(RwLock::new(std::time::Instant::now())))
    }
}

#[derive(Clone)]
pub(super) struct Subscription {
    pub(super) id: SubscriptionId,
    schedule: Arc<RwLock<Schedule>>,
    permits: Arc<AtomicUsize>,
    alive: Arc<AtomicBool>,
}

impl Subscription {
    pub(super) fn new() -> Self {
        Subscription {
            id: SubscriptionId::new(),
            schedule: Arc::new(RwLock::new(Schedule::default())),
            permits: Arc::new(AtomicUsize::new(0)),
            alive: Arc::new(AtomicBool::new(true)),
        }
    }

    pub(super) fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }

    pub(super) fn id(&self) -> SubscriptionId {
        self.id
    }

    pub(super) fn schedule(&self, schedule: impl Into<Schedule>) {
        *self.schedule.write().unwrap() = schedule.into();
    }

    pub(super) fn unsubscribe(&self) {
        self.alive.store(false, Ordering::Release);
        self.permits.store(0, Ordering::Release);
    }

    /// Called once per publish on every registration in a group, before anything is
    /// materialized — "is at least one subscriber in this group due?"
    pub(super) fn reserve(&self) -> bool {
        if !self.is_alive() {
            return false;
        }

        if !self.schedule.read().unwrap().is_scheduled() {
            return false;
        }

        self.permits.fetch_add(1, Ordering::Release);

        true
    }

    /// Called once per registration at actual delivery time — a schedule may have made
    /// several registrations "due" via `reserve`, but each only consumes its own permit.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn always_schedule_is_always_due() {
        let schedule = Schedule::Always;
        assert!(schedule.is_scheduled());
        assert!(schedule.is_scheduled());
    }

    #[test]
    fn every_n_schedule_is_due_only_on_the_nth_call() {
        let schedule = Schedule::from(3_usize);

        assert!(!schedule.is_scheduled());
        assert!(!schedule.is_scheduled());
        assert!(schedule.is_scheduled());
        assert!(!schedule.is_scheduled());
    }

    #[test]
    fn unsubscribe_marks_dead_and_clears_permits() {
        let subscription = Subscription::new();
        assert!(subscription.reserve());
        assert!(subscription.is_alive());

        subscription.unsubscribe();

        assert!(!subscription.is_alive());
        assert!(!subscription.take_permit());
        assert!(!subscription.reserve());
    }

    #[test]
    fn reserve_then_take_permit_round_trips() {
        let subscription = Subscription::new();

        assert!(subscription.reserve());
        assert!(subscription.take_permit());
        // Only one permit was reserved.
        assert!(!subscription.take_permit());
    }
}
