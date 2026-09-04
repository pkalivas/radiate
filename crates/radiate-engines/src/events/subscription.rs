use radiate_core::{AnyValue, EvalNoInput, Expr};
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
    Expr(Expr),
}

impl Schedule {
    pub fn is_scheduled(&mut self) -> bool {
        match self {
            Schedule::Always => true,
            Schedule::Expr(expr) => match expr.evaluate() {
                Ok(AnyValue::Bool(fired)) => fired,
                _ => false,
            },
        }
    }
}

impl From<usize> for Schedule {
    fn from(n: usize) -> Self {
        Schedule::Expr(Expr::every(n).into())
    }
}

impl From<Duration> for Schedule {
    fn from(duration: Duration) -> Self {
        Schedule::Expr(Expr::throttle(duration).into())
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
    pub(super) fn new() -> Self {
        Subscription {
            id: SubscriptionId::new(),
            schedule: Arc::new(RwLock::new(Schedule::default())),
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
        let mut guard = self.schedule.write().unwrap();
        guard.is_scheduled()
    }
}
