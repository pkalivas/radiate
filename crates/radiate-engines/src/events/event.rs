use crate::EventContext;
use radiate_core::{SmallStr, WaitGuard, error::RadiateResult};
use radiate_utils::{generate_metric_key, sentry_id};
use std::{any::Any, sync::Arc};

sentry_id!(EventId);

pub(super) type Payload = Arc<dyn Any + Send + Sync>;

pub trait Event: Send + Sync + 'static {
    fn name() -> SmallStr {
        const EVENT_PREFIX: &str = "event";
        SmallStr::from_string(generate_metric_key::<Self>(EVENT_PREFIX))
    }
}

pub trait Handler<E: Event>: Sized + Send + 'static {
    fn handle(&mut self, event: &E, ctx: &EventContext<'_, Self>);
}

pub trait EventHandler: Sized + Send + 'static {
    fn start(&mut self, _ctx: &EventContext<'_, Self>) -> RadiateResult<()> {
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

/// The chain an event belongs to: the id of the event that started it, and,
/// if something is waiting on the chain (see [`EventStream::flush`]), a guard
/// that keeps the wait open until every event in it has been handled.
///
/// [`EventStream::flush`]: crate::EventStream::flush
#[derive(Clone, Default)]
pub struct EventScope {
    id: EventId,
    // Never read: holding it is the point. It's released once the last
    // clone of this scope is dropped.
    _guard: Option<Arc<WaitGuard>>,
}

impl EventScope {
    /// A new chain that nothing is waiting on.
    pub(super) fn new() -> Self {
        EventScope {
            id: EventId::new(),
            _guard: None,
        }
    }

    /// A new chain that holds `guard` until it's finished.
    pub(super) fn guarded(guard: WaitGuard) -> Self {
        EventScope {
            id: EventId::new(),
            _guard: Some(Arc::new(guard)),
        }
    }

    pub fn id(&self) -> EventId {
        self.id
    }
}
