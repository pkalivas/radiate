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

#[derive(Clone, Default)]
pub enum Schedule {
    #[default]
    Always,
    EveryN(usize, Arc<AtomicUsize>),
    Duration(std::time::Duration, Arc<RwLock<std::time::Instant>>),
}

impl Schedule {
    pub fn is_scheduled(&self) -> bool {
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
pub struct Subscription {
    pub(crate) id: SubscriptionId,
    pub(crate) schedule: Arc<RwLock<Schedule>>,
    pub(crate) permits: Arc<AtomicUsize>,
    pub(crate) alive: Arc<AtomicBool>,
}

impl Subscription {
    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }

    pub fn id(&self) -> SubscriptionId {
        self.id
    }

    pub fn schedule(&mut self, schedule: impl Into<Schedule>) {
        *self.schedule.write().unwrap() = schedule.into();
    }

    pub fn unsubscribe(&self) {
        self.alive.store(false, Ordering::Release);
        self.permits.store(0, Ordering::Release);
    }

    pub(super) fn reserve(&self) -> bool {
        if !self.is_alive() {
            return false;
        }

        if !self.try_schedule() {
            return false;
        }

        self.permits.fetch_add(1, Ordering::Release);

        true
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

    fn try_schedule(&self) -> bool {
        self.schedule.read().unwrap().is_scheduled()
    }
}
