use crate::EventContext;
use radiate_core::{SmallStr, WaitGuard, error::RadiateResult};
use radiate_utils::{generate_metric_key, sentry_id};
use std::{
    any::Any,
    sync::{Arc, Weak},
};

sentry_id!(EventId);

pub(super) type Payload = Arc<dyn Any + Send + Sync>;

pub trait Event: Send + Sync + 'static {
    fn name() -> SmallStr {
        const EVENT_PREFIX: &str = "event";
        SmallStr::from_string(generate_metric_key::<Self>(EVENT_PREFIX))
    }
}

pub trait Handler<E: Event>: Send + 'static {
    fn handle(&mut self, event: &E, ctx: &EventContext<'_, Self>)
    where
        Self: Sized;
}

pub trait EventHandler: Send + 'static {
    fn start(&mut self, _ctx: &EventContext<'_, Self>) -> RadiateResult<()>
    where
        Self: Sized,
    {
        Ok(())
    }
}

impl<E, F> Handler<E> for F
where
    E: Event,
    F: FnMut(&E) + Send + 'static,
{
    fn handle(&mut self, event: &E, _ctx: &EventContext<'_, Self>) {
        self(event)
    }
}

// #[derive(Default, Clone)]
// pub struct Scope {
//     in_flight: Arc<()>,
//     #[allow(dead_code)]
//     guard: Option<Arc<WaitGuard>>,
// }

// impl Scope {
//     pub fn count(&self) -> usize {
//         Arc::strong_count(&self.in_flight)
//     }

//     pub fn guarded(&self, guard: WaitGuard) -> Scope {
//         Scope {
//             in_flight: Arc::clone(&self.in_flight),
//             guard: Some(Arc::new(guard)),
//         }
//     }
// }

#[derive(Default, Clone)]
pub struct StreamContext {
    in_flight: Arc<()>,
    #[allow(dead_code)]
    guard: Option<Arc<WaitGuard>>,
}

impl StreamContext {
    pub fn weak_count(&self) -> usize {
        Arc::weak_count(&self.in_flight)
    }

    pub fn weak(&self) -> WeakScope {
        WeakScope {
            in_flight: Arc::downgrade(&self.in_flight),
            guard: self.guard.clone(),
        }
    }

    pub fn guarded(&self, guard: WaitGuard) -> WeakScope {
        WeakScope {
            in_flight: Arc::downgrade(&self.in_flight),
            guard: Some(Arc::new(guard)),
        }
    }
}

#[derive(Default)]
pub struct WeakScope {
    in_flight: Weak<()>,
    #[allow(dead_code)]
    guard: Option<Arc<WaitGuard>>,
}

impl Clone for WeakScope {
    fn clone(&self) -> Self {
        WeakScope {
            in_flight: Weak::clone(&self.in_flight),
            guard: self.guard.clone(),
        }
    }
}
