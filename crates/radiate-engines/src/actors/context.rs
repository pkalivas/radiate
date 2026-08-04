use crate::{
    Actor, ActorRef, Executor, MessageHandler,
    actors::{actor::AnyActorRef, system::MessageBus},
};
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, RwLock},
};

type ErasedActorMap = HashMap<String, Box<dyn Any + Send + Sync>>;

#[derive(Default)]
pub struct ActorRegistry {
    registry: RwLock<ErasedActorMap>,
}

impl ActorRegistry {
    pub(super) fn insert<A: Actor + 'static>(&self, name: String, actor_ref: ActorRef<A>) {
        self.registry
            .write()
            .unwrap()
            .insert(name, Box::new(actor_ref));
    }

    pub(super) fn get<A: Actor + 'static>(&self, name: &str) -> Option<ActorRef<A>> {
        self.registry
            .read()
            .unwrap()
            .get(name)
            .and_then(|b| b.downcast_ref::<ActorRef<A>>().cloned())
    }
}

#[derive(Clone)]
pub struct ActorContext {
    pub(crate) executor: Arc<Executor>,
    pub(crate) bus: Arc<MessageBus>,
    pub(crate) registry: Arc<ActorRegistry>,
    pub(crate) parent: Option<AnyActorRef>,
}

impl ActorContext {
    pub fn executor(&self) -> Arc<Executor> {
        Arc::clone(&self.executor)
    }

    pub fn parent(&self) -> Option<AnyActorRef> {
        self.parent.clone()
    }

    pub fn bus(&self) -> Arc<MessageBus> {
        Arc::clone(&self.bus)
    }

    pub fn publish<M: Send + Clone + 'static>(&self, message: M) {
        self.bus.publish(message);
    }

    pub fn lazy_publish<M: Send + Clone + 'static>(&self, func: impl FnOnce() -> M) {
        if self.has_subscribers::<M>() {
            self.publish(func());
        }
    }

    pub fn has_subscribers<M: Send + 'static>(&self) -> bool {
        self.bus.has_subscribers::<M>()
    }

    /// Silently drops the message if `A` hasn't been spawned. Use when a
    /// missing recipient is a legitimate, expected state.
    pub fn tell<A, M>(&self, message: M)
    where
        A: MessageHandler<M> + 'static,
        M: Send + 'static,
    {
        if let Some(r) = self.actor::<A>() {
            r.tell(message);
        }
    }

    /// The actor registered under its own type name — the singleton case,
    /// where `spawn` (rather than `spawn_named`) chose the name for you.
    pub fn actor<A: Actor + 'static>(&self) -> Option<ActorRef<A>> {
        self.named::<A>(std::any::type_name::<A>())
    }

    pub fn named<A: Actor + 'static>(&self, name: &str) -> Option<ActorRef<A>> {
        self.registry.get::<A>(name)
    }
}
