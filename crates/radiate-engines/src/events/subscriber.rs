use crate::events::stream::Shared;
use crate::events::subscription::Counters;
use crate::events::{EventStream, Subscription, SubscriptionId};
use radiate_core::{Executor, SmallStr, WaitGuard, error::RadiateResult};
use radiate_utils::sentry_id;
use std::sync::{Arc, Mutex, Weak};

sentry_id!(EventId);

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
    /// A new, unscoped chain. Used by plain `publish` calls; nothing waits on it.
    pub(crate) fn new() -> Self {
        Trace {
            root: EventId::new(),
            scope: None,
        }
    }

    /// A new chain that holds `guard` until every event in it has been handled.
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

pub trait Event: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Event for T {}

pub trait Handler<E: Event>: Send + 'static {
    fn handle(&mut self, event: &E, ctx: &EventContext<'_, Self>)
    where
        Self: Sized;
}

pub trait EventHandler: Send + 'static {
    fn name(&self) -> SmallStr {
        let type_name = std::any::type_name::<Self>();
        SmallStr::from(type_name)
    }

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

pub struct EventContext<'a, H> {
    subscriber: &'a Subscriber<H>,
    trace: Trace,
}

impl<H> EventContext<'_, H> {
    /// Publish an event as part of the chain currently being handled. If that
    /// chain is being waited on (for example by [`EventStream::flush`]), the wait
    /// also covers this event and everything it causes.
    ///
    /// Does nothing if the stream has already been dropped.
    pub fn publish<E: Event>(&self, event: E) {
        if let Some(stream) = self.subscriber.stream() {
            stream.publish_traced(event, self.trace.clone());
        }
    }

    /// The id of the event that started the chain this handler is running in.
    pub fn root_id(&self) -> EventId {
        self.trace.root()
    }

    pub fn subscribe<E>(&self) -> Subscription
    where
        E: Event,
        H: Handler<E>,
    {
        self.subscriber.subscribe::<E>()
    }
}

pub struct Subscriber<H> {
    handler: Arc<Mutex<H>>,
    executor: Arc<Executor>,
    stream: Weak<Shared>,
}

impl<H> Subscriber<H> {
    /// The stream this subscriber belongs to, if it's still alive.
    fn stream(&self) -> Option<EventStream> {
        EventStream::upgrade(&self.executor, &self.stream)
    }
}

impl<H: Send + 'static> Subscriber<H> {
    pub(super) fn new(handler: H, executor: Arc<Executor>, stream: &EventStream) -> Self {
        Subscriber {
            handler: Arc::new(Mutex::new(handler)),
            executor,
            stream: stream.downgrade(),
        }
    }

    /// Subscribe this handler to `E`. If the stream has already been dropped,
    /// the returned subscription is inactive.
    pub fn subscribe<E>(&self) -> Subscription
    where
        E: Event,
        H: Handler<E>,
    {
        match self.stream() {
            Some(stream) => stream.subscribe_existing::<E, H>(self),
            None => {
                let subscription = Subscription::new();
                subscription.unsubscribe();
                subscription
            }
        }
    }

    pub fn unsubscribe(&self, id: SubscriptionId) {
        if let Some(stream) = self.stream() {
            stream.unsubscribe(id);
        }
    }

    pub(super) fn start(&self) -> RadiateResult<()>
    where
        H: EventHandler,
    {
        let ctx = EventContext {
            subscriber: self,
            trace: Trace::new(),
        };
        self.handler.lock().unwrap().start(&ctx)
    }

    /// Run the handler for `event`. The `trace` (and with it any scope guard) is
    /// kept alive until the handler returns, so a scope can't close while this
    /// handler, or anything it publishes, is still running.
    pub(super) fn send_shared<E>(&self, event: Arc<E>, counters: Arc<Counters>, trace: Trace)
    where
        E: Event,
        H: Handler<E>,
    {
        match self.executor.as_ref() {
            Executor::Serial => {
                let ctx = EventContext {
                    subscriber: self,
                    trace,
                };
                self.handler.lock().unwrap().handle(event.as_ref(), &ctx);
                counters.record_handled();
            }
            _ => {
                let owned = self.clone();
                self.executor.submit(move || {
                    let ctx = EventContext {
                        subscriber: &owned,
                        trace,
                    };
                    owned.handler.lock().unwrap().handle(event.as_ref(), &ctx);
                    counters.record_handled();
                });
            }
        }
    }
}

impl<H> Clone for Subscriber<H> {
    fn clone(&self) -> Self {
        Subscriber {
            handler: Arc::clone(&self.handler),
            executor: Arc::clone(&self.executor),
            stream: Weak::clone(&self.stream),
        }
    }
}
