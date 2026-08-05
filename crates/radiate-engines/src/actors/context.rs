use crate::{
    Actor, Addr, Executor,
    actors::{
        ProcessId,
        actor::{ActorCtx, WeakAddr},
        system::MessageBus,
    },
};
use std::{
    any::Any,
    collections::HashMap,
    sync::{Arc, Mutex, RwLock, Weak, atomic::AtomicBool},
};

type ErasedActorMap = HashMap<ProcessId, Box<dyn Any + Send + Sync>>;

#[derive(Default)]
pub struct ActorRegistry {
    registry: RwLock<ErasedActorMap>,
}

impl ActorRegistry {
    pub(super) fn insert<A: Actor + 'static>(&self, name: ProcessId, actor_ref: Addr<A>) {
        self.registry
            .write()
            .unwrap()
            .insert(name, Box::new(actor_ref));
    }

    pub(super) fn get<A: Actor + 'static>(&self, name: &ProcessId) -> Option<Addr<A>> {
        self.registry
            .read()
            .unwrap()
            .get(name)
            .and_then(|b| b.downcast_ref::<Addr<A>>().cloned())
    }

    pub(super) fn remove<A: Actor + 'static>(&self, name: &ProcessId) -> Option<Addr<A>> {
        self.registry
            .write()
            .unwrap()
            .remove(name)
            .and_then(|b| b.downcast::<Addr<A>>().ok().map(|b| *b))
    }
}

#[derive(Clone)]
pub struct SystemCtx {
    pub(crate) executor: Arc<Executor>,
    pub(crate) bus: Arc<MessageBus>,
    pub(crate) registry: Arc<ActorRegistry>,
}

impl SystemCtx {
    pub fn executor(&self) -> Arc<Executor> {
        Arc::clone(&self.executor)
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

    /// The actor registered under its own type name — the singleton case,
    /// where `spawn` (rather than `spawn_named`) chose the name for you.
    pub fn actor<A: Actor + 'static>(&self, pid: impl Into<ProcessId>) -> Option<Addr<A>> {
        self.registry.get::<A>(&pid.into())
    }

    pub fn create<A, F>(&self, pid: ProcessId, f: F) -> Addr<A>
    where
        A: Actor + 'static,
        F: FnOnce(&WeakAddr<A>) -> A,
    {
        let (sender, receiver) = std::sync::mpsc::channel();
        let context = SystemCtx {
            bus: Arc::clone(&self.bus),
            executor: Arc::clone(&self.executor),
            registry: Arc::clone(&self.registry),
        };

        let cell = Arc::new_cyclic(|weak: &Weak<ActorCtx<A>>| {
            let self_ref = WeakAddr {
                cell: Weak::clone(weak),
            };
            let actor = f(&self_ref);

            ActorCtx {
                pid,
                actor: Arc::new(Mutex::new(actor)),
                sender,
                receiver,
                scheduled: AtomicBool::new(false),
                stopped: AtomicBool::new(false),
                parent: None,
                context: context.clone(),
            }
        });

        Addr { cell }
    }
}
