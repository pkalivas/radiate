use crate::{
    EventHandler, Executor, MessageHandler, SystemCtx,
    actors::{
        DeadLetter, ProcessId,
        actor::{Actor, Addr, Recipient, WeakAddr},
        context::ActorRegistry,
        handler::FnActor,
        message::{ActorRegistered, DeadLetterActor},
    },
};
use radiate_core::SmallStr;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

const DEFAULT_ACTOR_SYSTEM_NAME: ProcessId =
    ProcessId::new_const(SmallStr::from_static("actor-system"));

#[derive(Clone)]
pub struct ActorSystem {
    pid: ProcessId,
    context: SystemCtx,
}

impl ActorSystem {
    pub fn new(pid: ProcessId, executor: Arc<Executor>) -> Self {
        ActorSystem {
            pid,
            context: SystemCtx {
                executor,
                bus: Arc::new(MessageBus::default()),
                registry: Arc::new(ActorRegistry::default()),
            },
        }
    }

    pub fn set_bus(mut self, bus: Arc<MessageBus>) -> Self {
        self.context.bus = bus;
        self
    }

    pub fn start(&self) {
        self.spawn_fn("dead-letter-queue", || DeadLetterActor::default());
    }

    pub fn stop(&self) {
        let keys = self.context.registry.keys();
        for key in keys {
            if let Some(stop_hook) = self.context.registry.get_stop_hook(&key) {
                let actor_report = stop_hook();
                self.context.registry.remove(&actor_report.pid);
            }
        }
    }

    pub fn pid(&self) -> &ProcessId {
        &self.pid
    }

    pub fn context(&self) -> SystemCtx {
        self.context.clone()
    }

    pub fn publish<M: Send + Clone + 'static>(&self, message: M) {
        self.context.bus.publish(message);
    }

    pub fn send<A, M>(&self, pid: impl Into<ProcessId>, message: M)
    where
        A: MessageHandler<M> + 'static,
        M: Send + Clone + 'static,
    {
        let pid = pid.into();
        if let Some(addr) = self.context.registry.get::<A>(&pid) {
            addr.send(message);
        } else {
            self.publish(DeadLetter {
                pid,
                message_type: std::any::type_name::<M>(),
            });
        }
    }

    pub fn spawn<A: Actor + 'static>(&self, actor: A) -> Addr<A> {
        self.spawn_named(std::any::type_name::<A>(), actor)
    }

    pub fn spawn_named<A: Actor + 'static>(&self, name: impl Into<ProcessId>, actor: A) -> Addr<A> {
        let pid = name.into();
        let actor_ref = self.context.create(pid.clone(), |_: &WeakAddr<A>| actor);

        self.register_actor(pid, actor_ref.clone());
        actor_ref.cell.actor.lock().unwrap().on_init(&actor_ref);

        actor_ref
    }

    pub fn spawn_fn<A, F>(&self, pid: impl Into<ProcessId>, f: F) -> Addr<A>
    where
        A: Actor + 'static,
        F: FnOnce() -> A,
    {
        let pid = pid.into();
        let actor = f();
        let addr = self.context.create(pid.clone(), |_: &WeakAddr<A>| actor);

        addr.cell.actor.lock().unwrap().on_init(&addr);
        self.register_actor(pid, addr.clone());

        addr
    }

    pub fn subscribe<M: Send + Clone + 'static>(
        &self,
        mut handler: impl EventHandler<M> + Send + Sync + 'static,
    ) {
        self.subscribe_with::<M, _>(move |message, _| handler.handle(&message));
    }

    fn subscribe_with<M, F>(&self, f: F)
    where
        M: Send + Clone + 'static,
        F: FnMut(M, &Addr<FnActor<M>>) + Send + Sync + 'static,
    {
        let actor = FnActor {
            handler: Box::new(f),
        };

        let pid = ProcessId::from(format!("subscriber-{}", std::any::type_name::<M>()));
        let actor_ref = self.spawn_named(pid.clone(), actor);

        actor_ref.cell.actor.lock().unwrap().on_init(&actor_ref);

        self.context.bus.subscribe(actor_ref.recipient::<M>());
        self.register_actor(pid, actor_ref.clone());
    }

    fn register_actor<A: Actor + 'static>(&self, pid: ProcessId, addr: Addr<A>) {
        self.context.registry.insert(pid.clone(), addr);
        self.publish(ActorRegistered { pid });
    }
}

impl From<(Arc<Executor>, Arc<MessageBus>)> for ActorSystem {
    fn from((executor, bus): (Arc<Executor>, Arc<MessageBus>)) -> Self {
        ActorSystem::new(DEFAULT_ACTOR_SYSTEM_NAME, executor).set_bus(bus)
    }
}

impl Default for ActorSystem {
    fn default() -> Self {
        ActorSystem::new(DEFAULT_ACTOR_SYSTEM_NAME, Arc::new(Executor::default()))
    }
}

#[derive(Default)]
pub struct MessageBus {
    subscribers: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
}

impl MessageBus {
    pub fn subscribe<M: Send + 'static>(&self, recipient: Recipient<M>) {
        let mut registry = self.subscribers.write().unwrap();

        registry
            .entry(TypeId::of::<M>())
            .or_insert_with(|| Box::new(Vec::<Recipient<M>>::new()) as Box<dyn Any + Send + Sync>)
            .downcast_mut::<Vec<Recipient<M>>>()
            .expect("TypeId key always matches Vec<Recipient<M>> by construction")
            .push(recipient);
    }

    pub fn publish<M: Send + Clone + 'static>(&self, message: M) {
        let registry = self.subscribers.read().unwrap();
        if let Some(group) = registry
            .get(&TypeId::of::<M>())
            .and_then(|b| b.downcast_ref::<Vec<Recipient<M>>>())
        {
            for recipient in group {
                recipient.tell(message.clone());
            }
        }
    }

    pub fn has_subscribers<M: Send + 'static>(&self) -> bool {
        self.subscribers
            .read()
            .unwrap()
            .contains_key(&TypeId::of::<M>())
    }
}
