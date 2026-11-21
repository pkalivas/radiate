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

#[derive(Default, Clone)]
pub struct StreamContext {
    in_flight: Arc<()>,
    #[allow(dead_code)]
    guard: Option<Arc<WaitGuard>>,
}

impl StreamContext {
    #[allow(dead_code)]
    pub fn in_flight(&self) -> usize {
        Arc::weak_count(&self.in_flight)
    }

    pub fn weak(&self) -> EventScope {
        EventScope {
            id: EventId::new(),
            in_flight: Arc::downgrade(&self.in_flight),
            guard: self.guard.clone(),
        }
    }

    pub fn guarded(&self, guard: WaitGuard) -> EventScope {
        EventScope {
            id: EventId::new(),
            in_flight: Arc::downgrade(&self.in_flight),
            guard: Some(Arc::new(guard)),
        }
    }
}

#[derive(Default)]
pub struct EventScope {
    id: EventId,
    in_flight: Weak<()>,
    #[allow(dead_code)]
    guard: Option<Arc<WaitGuard>>,
}

impl EventScope {
    pub fn id(&self) -> EventId {
        self.id
    }
}

impl Clone for EventScope {
    fn clone(&self) -> Self {
        EventScope {
            id: self.id,
            in_flight: Weak::clone(&self.in_flight),
            guard: self.guard.clone(),
        }
    }
}
