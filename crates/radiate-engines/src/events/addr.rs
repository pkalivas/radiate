use super::cell::ActorCell;
use crate::events::{Event, EventStream, Subscription, SubscriptionId};
use crossbeam::channel;
use radiate_core::{Executor, RadiateError, SmallStr, error::RadiateResult};
use radiate_utils::sentry_id;
use std::sync::Arc;

sentry_id!(ActorId);

pub trait Message: Send + 'static {
    type Response: Send + 'static;
}

pub trait MessageHandler<M: Message>: Actor {
    fn handle(&mut self, msg: &M, ctx: &ActorContext<Self>) -> M::Response
    where
        Self: Sized;
}

pub trait Actor: Send + 'static {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn started(&mut self, _ctx: &ActorContext<Self>)
    where
        Self: Sized,
    {
    }

    fn stopped(&mut self)
    where
        Self: Sized,
    {
    }
}

pub(super) type ProcessFn<A> = Box<dyn FnOnce(&mut A, &ActorContext<A>) + Send>;

pub(super) struct Envelope<A> {
    pub(super) run: ProcessFn<A>,
}

pub struct ActorContext<A>(pub(super) Addr<A>);

impl<A: Actor> ActorContext<A> {
    pub fn send<M>(&self, msg: M)
    where
        M: Message,
        A: MessageHandler<M>,
    {
        (self.0).send(msg);
    }

    pub fn ask<M>(&self, msg: M) -> RadiateResult<M::Response>
    where
        M: Message,
        A: MessageHandler<M>,
    {
        (self.0).ask(msg)
    }

    pub fn publish<E: Event>(&self, message: E) {
        (self.0).publish(message);
    }

    pub fn subscribe<E>(&self) -> Option<Subscription>
    where
        E: Event,
        A: MessageHandler<E>,
    {
        if let Some(bus) = &self.0.bus {
            Some(bus.subscribe_addr::<E, A>(&self.0))
        } else {
            None
        }
    }
}

pub struct Addr<A> {
    pub(super) name: SmallStr,
    pub(super) id: ActorId,
    pub(super) cell: Arc<ActorCell<A>>,
    pub(super) executor: Arc<Executor>,
    pub(super) bus: Option<EventStream>,
}

impl<A: Actor> Addr<A> {
    pub fn new(actor: A, executor: Arc<Executor>, bus: Option<EventStream>) -> Self {
        let name = actor.name().into();
        let cell = Arc::new(ActorCell::new(actor));
        let addr = Addr {
            name,
            id: ActorId::new(),
            cell,
            executor,
            bus,
        };

        {
            let mut guard = addr.cell.actor.lock().unwrap_or_else(|e| e.into_inner());
            let ctx = ActorContext(addr.clone());

            guard.started(&ctx);
        }

        addr
    }

    pub fn name(&self) -> SmallStr {
        self.name.clone()
    }

    pub fn send_shared<E>(&self, message: Arc<E>)
    where
        E: Event,
        A: MessageHandler<E>,
    {
        let queued = self.cell.enqueue(Envelope {
            run: Box::new(move |actor: &mut A, ctx: &ActorContext<A>| {
                actor.handle(message.as_ref(), ctx);
            }),
        });

        if queued {
            self.dispatch();
        }
    }

    pub fn send<M>(&self, msg: M)
    where
        M: Message,
        A: MessageHandler<M>,
    {
        let queued = self.cell.enqueue(Envelope {
            run: Box::new(move |actor: &mut A, ctx: &ActorContext<A>| {
                actor.handle(&msg, ctx);
            }),
        });

        if queued {
            self.dispatch();
        }
    }

    pub fn ask<M>(&self, msg: M) -> RadiateResult<M::Response>
    where
        M: Message,
        A: MessageHandler<M>,
    {
        let (tx, rx) = channel::bounded(1);

        let queued = self.cell.enqueue(Envelope {
            run: Box::new(move |actor: &mut A, ctx: &ActorContext<A>| {
                let response = actor.handle(&msg, ctx);
                // Caller may have dropped `rx` already (unlikely, but not
                // our problem if so) — don't let that panic the actor.
                let _ = tx.send(response);
            }),
        });

        if !queued {
            // Actor has stopped, so the caller will never get a response.
            return Err(RadiateError::Event(format!(
                "actor stopped before responding"
            )));
        }

        self.dispatch();
        rx.recv()
            .map_err(|_| RadiateError::Event(format!("Failed to receive response from actor")))
    }

    pub fn publish<E: Event>(&self, message: E) {
        if let Some(bus) = &self.bus {
            bus.publish(message);
        }
    }

    pub fn subscribe<E>(&self) -> Option<Subscription>
    where
        E: Event,
        A: MessageHandler<E>,
    {
        let bus = self.bus.as_ref()?;
        let subscription = bus.subscribe_addr::<E, A>(self);

        Some(subscription)
    }

    pub fn unsubscribe<E: 'static>(&self, id: SubscriptionId) {
        if let Some(bus) = &self.bus {
            bus.unsubscribe(id);
        }
    }

    fn dispatch(&self) {
        if self.cell.try_claim() {
            let cell = Arc::clone(&self.cell);
            let addr = self.clone();
            self.executor.submit(move || cell.process_batch(addr));
        }
    }
}

impl<A> Clone for Addr<A> {
    fn clone(&self) -> Self {
        Addr {
            name: self.name.clone(),
            id: self.id,
            cell: Arc::clone(&self.cell),
            executor: Arc::clone(&self.executor),
            bus: self.bus.clone(),
        }
    }
}

impl<A: Actor> From<(A, Arc<Executor>)> for Addr<A> {
    fn from((actor, executor): (A, Arc<Executor>)) -> Self {
        Addr::new(actor, executor, None)
    }
}

impl<A: Actor> From<(A, Arc<Executor>, EventStream)> for Addr<A> {
    fn from((actor, executor, bus): (A, Arc<Executor>, EventStream)) -> Self {
        Addr::new(actor, executor, Some(bus))
    }
}
