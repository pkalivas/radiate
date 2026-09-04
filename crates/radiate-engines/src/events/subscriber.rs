use crate::events::{EventStream, Subscription, SubscriptionId};
use radiate_core::{Executor, error::RadiateResult};
use std::sync::{Arc, Mutex};

pub trait Event: Send + Sync + 'static {
    fn event_label() -> &'static str {
        std::any::type_name::<Self>()
    }
}
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

pub struct EventContext<'a, H>(&'a Subscriber<H>);

impl<H> EventContext<'_, H> {
    pub fn publish<E: Event>(&self, event: E) {
        self.0.stream.publish(event);
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
        let ctx = EventContext(self);
        self.handler.lock().unwrap().start(&ctx)
    }

    pub(super) fn send_shared<E>(&self, event: Arc<E>)
    where
        E: Event,
        H: Handler<E>,
    {
        match self.executor.as_ref() {
            Executor::Serial => {
                let ctx = EventContext(self);
                self.handler.lock().unwrap().handle(event.as_ref(), &ctx);
            }
            _ => {
                let owned = self.clone();
                self.executor.submit(move || {
                    let ctx = EventContext(&owned);
                    owned.handler.lock().unwrap().handle(event.as_ref(), &ctx);
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
