use crate::events::subscription::Counters;
use crate::events::{EventStream, Subscription, SubscriptionId};
use crossbeam::epoch::Guard;
use radiate_core::{Executor, WaitGuard, error::RadiateResult};
use radiate_utils::sentry_id;
use std::sync::{Arc, Mutex};

sentry_id!(EventId);

pub trait Event: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Event for T {}

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

pub struct EventContext<'a, H>(&'a Subscriber<H>, Option<Arc<WaitGuard>>);

impl<H> EventContext<'_, H> {
    pub fn publish<E: Event>(&self, event: E) {
        match &self.1 {
            Some(guard) => self.0.stream.relay(event, Arc::clone(guard)),
            None => self.0.stream.publish(event),
        }
    }

    pub fn subscribe<E>(&self) -> Subscription
    where
        E: Event,
        H: Handler<E>,
    {
        self.0.subscribe::<E>()
    }
}

pub struct Subscriber<H> {
    handler: Arc<Mutex<H>>,
    executor: Arc<Executor>,
    stream: EventStream,
}

impl<H: Send + 'static> Subscriber<H> {
    pub(super) fn new(handler: H, executor: Arc<Executor>, stream: EventStream) -> Self {
        Subscriber {
            handler: Arc::new(Mutex::new(handler)),
            executor,
            stream,
        }
    }

    pub fn subscribe<E>(&self) -> Subscription
    where
        E: Event,
        H: Handler<E>,
    {
        self.stream.subscribe_existing::<E, H>(self)
    }

    pub fn unsubscribe(&self, id: SubscriptionId) {
        self.stream.unsubscribe(id);
    }

    pub(super) fn start(&self) -> RadiateResult<()>
    where
        H: EventHandler,
    {
        let ctx = EventContext(self, None);
        self.handler.lock().unwrap().start(&ctx)
    }

    pub(super) fn send_shared<E>(
        &self,
        event: Arc<E>,
        counters: Arc<Counters>,
        guard: Arc<WaitGuard>,
    ) where
        E: Event,
        H: Handler<E>,
    {
        match self.executor.as_ref() {
            Executor::Serial => {
                let ctx = EventContext(self, Some(guard.clone()));
                self.handler.lock().unwrap().handle(event.as_ref(), &ctx);
                drop(guard);
                counters.record_handled();
            }
            _ => {
                let owned = self.clone();
                self.executor.submit(move || {
                    let ctx = EventContext(&owned, Some(guard.clone()));
                    owned.handler.lock().unwrap().handle(event.as_ref(), &ctx);
                    drop(guard);
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
            stream: self.stream.clone(),
        }
    }
}
