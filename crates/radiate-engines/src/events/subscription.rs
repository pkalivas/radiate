use radiate_core::{AnyValue, Expr};
use radiate_utils::sentry_id;
use std::{
    fmt::Debug,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};
use std::{sync::atomic::AtomicUsize, time::Duration};

sentry_id!(SubscriptionId);

#[derive(Clone, Default)]
pub enum Schedule {
    #[default]
    Always,
    EveryN(Arc<RwLock<Expr>>),
    Duration(Arc<RwLock<Expr>>),
}

impl Schedule {
    pub fn is_scheduled(&self) -> bool {
        match self {
            Schedule::Always => true,
            Schedule::EveryN(counter) => match counter.write().unwrap().tick() {
                Ok(AnyValue::Bool(fired)) => fired,
                _ => false,
            },
            Schedule::Duration(last_time) => match last_time.write().unwrap().tick() {
                Ok(AnyValue::Bool(fired)) => fired,
                _ => false,
            },
        }
    }
}

impl From<usize> for Schedule {
    fn from(n: usize) -> Self {
        Schedule::EveryN(Arc::new(RwLock::new(
            Expr::every(n).then(true).otherwise(false),
        )))
    }
}

impl From<Duration> for Schedule {
    fn from(duration: Duration) -> Self {
        Schedule::Duration(Arc::new(RwLock::new(Expr::throttle(duration))))
    }
}

#[derive(Clone)]
pub struct Subscription {
    pub(crate) id: SubscriptionId,
    pub(crate) schedule: Arc<RwLock<Option<Schedule>>>,
    pub(crate) permits: Arc<AtomicUsize>,
    pub(crate) alive: Arc<AtomicBool>,
}

impl Subscription {
    pub(super) fn new() -> Self {
        Subscription {
            id: SubscriptionId::new(),
            schedule: Arc::new(RwLock::new(Some(Schedule::default()))),
            permits: Arc::new(AtomicUsize::new(0)),
            alive: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }

    pub fn id(&self) -> SubscriptionId {
        self.id
    }

    pub fn schedule(&self, schedule: impl Into<Schedule>) {
        *self.schedule.write().unwrap() = Some(schedule.into());
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
        if let Some(inner) = &*self.schedule.read().unwrap() {
            inner.is_scheduled()
        } else {
            false
        }
    }
}
