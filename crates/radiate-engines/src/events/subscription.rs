use radiate_core::{EvalNoInput, Expr, RadiateError, error::RadiateResult, radiate_err};
use radiate_error::radiate_bail;
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
pub struct Subscription {
    pub(crate) id: SubscriptionId,
    pub(crate) schedule: Arc<RwLock<Option<Expr>>>,
    pub(crate) permits: Arc<AtomicUsize>,
    pub(crate) alive: Arc<AtomicBool>,
}

impl Subscription {
    pub(super) fn new() -> Self {
        Subscription {
            id: SubscriptionId::new(),
            schedule: Arc::new(RwLock::new(None)),
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

    pub fn schedule(&self, schedule: impl Into<Expr>) -> Result<bool, RadiateError> {
        let maybe_expr = schedule.into();

        match maybe_expr.clone().into_schedule() {
            Some(expr) => {
                *self.schedule.write().unwrap() = Some(expr);
                return Ok(true);
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
            return Ok(false);
        }

        self.permits.fetch_add(1, Ordering::Release);

        Ok(true)
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
        let mut guard = self.schedule.write().unwrap();
        if let Some(expr) = &mut *guard {
            return expr
                .compute()?
                .extract_bool()
                .ok_or_else(|| radiate_err!(Expr: "Failed to compute schedule as bool"));
        }

        return Ok(false);
    }
}
