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

/// The causal chain an event belongs to.
///
/// Every event published from inside a handler (through its [`EventContext`])
/// inherits the trace of the event being handled, so all events in a chain share
/// the same `root` id. When the chain is `scoped`, each event in it holds a clone
/// of the scope's [`WaitGuard`]. The guard is released only once every handler
/// in the chain has returned, which lets the code that started the chain wait
/// for it to finish (see [`EventStream::flush`]).
#[derive(Clone)]
pub(crate) struct Trace {
    root: EventId,
    // Never read: only held, so its Drop releases the scope once the chain is done.
    #[allow(dead_code)]
    scope: Option<Arc<WaitGuard>>,
}

impl Trace {
    pub(crate) fn new() -> Self {
        Trace {
            root: EventId::new(),
            scope: None,
        }
    }

    pub(crate) fn scoped(guard: WaitGuard) -> Self {
        Trace {
            root: EventId::new(),
            scope: Some(Arc::new(guard)),
        }
    }

    pub(crate) fn root(&self) -> EventId {
        self.root
    }
}
