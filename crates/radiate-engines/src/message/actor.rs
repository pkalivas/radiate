use super::cell::ActorCell;
use crate::message::{Event, EventStream, Subscription, SubscriptionId};
use crossbeam::channel;
use radiate_core::{Executor, RadiateError, error::RadiateResult};
use radiate_utils::sentry_id;
use std::{
    any::TypeId,
    collections::HashMap,
    sync::{Arc, RwLock},
};

sentry_id!(ActorId);

pub trait Message: Send + Sync + 'static {
    type Response: Send + 'static;
}

impl<T: Event> Message for Arc<T> {
    type Response = ();
}

pub trait MessageHandler<M: Message>: Actor {
    fn handle(&mut self, msg: M, ctx: &ActorContext<Self>) -> M::Response
    where
        Self: Sized;
}

pub trait Actor: Send + Sync + 'static {
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
        A: MessageHandler<M>,
        M: Message,
    {
        (self.0).send(msg);
    }

    pub fn ask<M>(&self, msg: M) -> RadiateResult<M::Response>
    where
        A: MessageHandler<M>,
        M: Message,
    {
        (self.0).ask(msg)
    }

    pub fn publish<E: Event>(&self, message: E) {
        (self.0).publish(message);
    }
}

pub struct Addr<A> {
    pub(super) id: ActorId,
    pub(super) cell: Arc<ActorCell<A>>,
    pub(super) executor: Arc<Executor>,
    pub(super) bus: Option<EventStream>,
    pub(super) subscriptions: Arc<RwLock<HashMap<TypeId, SubscriptionId>>>,
}

impl<A: Actor> Addr<A> {
    pub fn spawn(actor: A, executor: Arc<Executor>) -> Self {
        Self::spawn_with_bus(actor, executor, None)
    }

    pub fn spawn_with_bus(actor: A, executor: Arc<Executor>, bus: Option<EventStream>) -> Self {
        let cell = Arc::new(ActorCell::new(actor));
        let addr = Addr {
            id: ActorId::new(),
            cell,
            executor,
            bus,
            subscriptions: Arc::default(),
        };

        {
            let mut guard = addr.cell.actor.lock().unwrap_or_else(|e| e.into_inner());
            let ctx = ActorContext(addr.clone());

            guard.started(&ctx);
        }

        addr
    }

    pub fn unsubscribe<E>(&self, id: SubscriptionId)
    where
        E: Event,
    {
        if let Some(bus) = &self.bus {
            let mut subscriptions = self.subscriptions.write().unwrap();
            subscriptions.remove(&TypeId::of::<E>());
            bus.unsubscribe(id);
        }
    }

    pub fn send<M>(&self, msg: M)
    where
        A: MessageHandler<M>,
        M: Message,
    {
        let queued = self.cell.enqueue(Envelope {
            run: Box::new(move |actor: &mut A, ctx: &ActorContext<A>| {
                actor.handle(msg, ctx);
            }),
        });

        if queued {
            self.dispatch();
        }
    }

    pub fn ask<M>(&self, msg: M) -> RadiateResult<M::Response>
    where
        A: MessageHandler<M>,
        M: Message,
    {
        let (tx, rx) = channel::bounded(1);

        let queued = self.cell.enqueue(Envelope {
            run: Box::new(move |actor: &mut A, ctx: &ActorContext<A>| {
                let response = actor.handle(msg, ctx);
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

    pub fn receive<E>(&self) -> Option<Subscription>
    where
        E: Event,
        A: MessageHandler<Arc<E>>,
    {
        self.bus
            .as_ref()
            .map(|bus| bus.subscribe_addr::<E, A>(self))
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
            id: self.id,
            cell: Arc::clone(&self.cell),
            executor: Arc::clone(&self.executor),
            bus: self.bus.clone(),
            subscriptions: Arc::clone(&self.subscriptions),
        }
    }
}
