use crate::{
    Actor,
    message::actor::{ActorContext, Addr, MessageHandler},
};
use radiate_core::Executor;
use radiate_utils::sentry_id;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    marker::PhantomData,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

sentry_id!(SubscriptionId);

pub trait Event: Send + Sync + 'static {
    fn event_label() -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T: Send + Sync + 'static> Event for T {}

pub trait EventHandler<E>: Send + Sync + 'static {
    fn handle(&mut self, event: &E);
}

impl<E, F> EventHandler<E> for F
where
    E: Event,
    F: FnMut(&E) + Send + Sync + 'static,
{
    fn handle(&mut self, event: &E) {
        self(event)
    }
}

pub struct Subscription {
    id: SubscriptionId,
    active: Arc<AtomicBool>,
}

impl Subscription {
    pub fn unsubscribe(&self) {
        self.active.store(false, Ordering::Release);
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    pub fn id(&self) -> SubscriptionId {
        self.id
    }
}

type Payload = Arc<dyn Any + Send + Sync>;
type Forward = Arc<dyn Fn(Payload) + Send + Sync>;

#[derive(Clone)]
struct Registration {
    id: SubscriptionId,
    forward: Forward,
    active: Arc<AtomicBool>,
}

type SubscriberList = Arc<Vec<Registration>>;
type SubscriberMap = HashMap<TypeId, SubscriberList>;

#[derive(Clone, Default)]
pub struct EventStream {
    executor: Arc<Executor>,
    subscribers: Arc<RwLock<SubscriberMap>>,
}

impl EventStream {
    pub fn new(executor: Arc<Executor>) -> Self {
        EventStream {
            executor,
            subscribers: Arc::default(),
        }
    }

    pub fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
    }

    pub fn spawn<A: Actor>(&self, actor: A) -> Addr<A> {
        Addr::spawn_with_bus(actor, Arc::clone(&self.executor), Some(self.clone()))
    }

    pub fn unsubscribe(&self, id: SubscriptionId) {
        let mut subscribers = self.subscribers.write().unwrap();
        for group in subscribers.values_mut() {
            let mut_group = Arc::make_mut(group);
            mut_group.retain(
                |registration| {
                    if registration.id == id { false } else { true }
                },
            );
        }
    }

    pub fn subscribe<E: Event>(&self, handler: impl EventHandler<E>) -> Subscription {
        let addr = Addr::spawn(
            EventForwardActor {
                handler,
                _marker: PhantomData,
            },
            Arc::clone(&self.executor),
        );
        self.subscribe_addr::<E, _>(&addr)
    }

    pub fn subscribe_addr<E, A>(&self, addr: &Addr<A>) -> Subscription
    where
        E: Event,
        A: MessageHandler<Arc<E>>,
    {
        let active = Arc::new(AtomicBool::new(true));
        let id = SubscriptionId::new();
        let addr = addr.clone();

        let forward = Arc::new(move |payload: Payload| {
            if let Ok(msg) = payload.downcast::<E>() {
                addr.send(msg);
            }
        });

        self.register(
            TypeId::of::<E>(),
            Registration {
                id,
                forward,
                active: Arc::clone(&active),
            },
        );

        Subscription { id, active }
    }

    #[inline]
    pub fn publish<E: Event>(&self, message: E) {
        let group = {
            let subscribers = self.subscribers.read().unwrap();
            match subscribers.get(&TypeId::of::<E>()) {
                Some(group) => Arc::clone(group),
                None => return,
            }
        };

        let payload: Payload = Arc::new(message);

        for registration in group.iter() {
            if !registration.active.load(Ordering::Acquire) {
                continue;
            }

            (registration.forward)(Arc::clone(&payload));
        }
    }

    #[inline]
    pub fn lazy_publish<E: Event>(&self, f: impl FnOnce() -> E) {
        if self.can_publish::<E>() {
            self.publish(f());
        }
    }

    #[inline]
    pub fn handler_count<E: Event>(&self) -> usize {
        self.subscribers
            .read()
            .unwrap()
            .get(&TypeId::of::<E>())
            .map(|group| {
                group
                    .iter()
                    .filter(|registration| registration.active.load(Ordering::Acquire))
                    .count()
            })
            .unwrap_or(0)
    }

    fn register(&self, type_id: TypeId, registration: Registration) {
        let mut subscribers = self.subscribers.write().unwrap();

        let list = subscribers.entry(type_id).or_default();

        let registrations = Arc::make_mut(list);
        registrations.retain(|registration| registration.active.load(Ordering::Acquire));
        registrations.push(registration);
    }

    fn can_publish<E: Event>(&self) -> bool {
        self.handler_count::<E>() > 0
    }
}

impl Debug for EventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let subscribers = self.subscribers.read().unwrap();
        let mut registrations = String::new();

        for (type_id, group) in subscribers.iter() {
            for registration in group.iter() {
                registrations.push_str(&format!(
                    "TypeId={:?}, id={}, active={}\n",
                    type_id,
                    registration.id,
                    registration.active.load(Ordering::Acquire)
                ));
            }
        }

        write!(
            f,
            "EventStream(subscribers={}, executor={:?}, registrations=\n{})",
            subscribers.len(),
            self.executor,
            registrations,
        )
    }
}

struct EventForwardActor<M, H> {
    handler: H,
    _marker: PhantomData<fn(&M)>,
}

impl<M, H> Actor for EventForwardActor<M, H>
where
    M: Event,
    H: EventHandler<M>,
{
}

impl<M, H> MessageHandler<Arc<M>> for EventForwardActor<M, H>
where
    M: Event,
    H: EventHandler<M>,
{
    fn handle(&mut self, msg: Arc<M>, _: &ActorContext<Self>) {
        self.handler.handle(&msg);
    }
}
