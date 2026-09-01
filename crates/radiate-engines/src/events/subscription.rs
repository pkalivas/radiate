use radiate_utils::sentry_id;
use std::sync::atomic::AtomicUsize;
use std::{
    fmt::Debug,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

sentry_id!(SubscriptionId);

#[derive(Clone)]
pub enum Schedule {
    Always,
    EveryN(usize, Arc<AtomicUsize>),
}

impl Schedule {
    pub fn is_scheduled(&self) -> bool {
        match self {
            Schedule::Always => true,
            Schedule::EveryN(n, counter) => {
                let current = counter.fetch_add(1, Ordering::Relaxed);
                current.saturating_add(1).is_multiple_of(*n)
            }
        }
    }
}

impl Default for Schedule {
    fn default() -> Self {
        Schedule::Always
    }
}

impl From<usize> for Schedule {
    fn from(n: usize) -> Self {
        Schedule::EveryN(n, Arc::new(AtomicUsize::new(0)))
    }
}

#[derive(Clone)]
pub struct Subscription {
    pub(crate) id: SubscriptionId,
    pub(crate) active: Arc<AtomicBool>,
    pub(crate) schedule: Arc<RwLock<Schedule>>,
}

impl Subscription {
    pub fn unsubscribe(&self) {
        self.active.store(false, Ordering::Release);
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub fn id(&self) -> SubscriptionId {
        self.id
    }

    pub fn schedule(&mut self, schedule: impl Into<Schedule>) {
        *self.schedule.write().unwrap() = schedule.into();
    }

    pub(super) fn try_schedule(&self) -> bool {
        self.schedule.read().unwrap().is_scheduled()
    }
}
