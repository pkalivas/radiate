use crate::{
    Actor, SmallStr,
    message::actor::{ActorContext, ActorId, Addr, Message, MessageHandler},
};
use radiate_core::Executor;
use radiate_utils::sentry_id;
use std::sync::Mutex;
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

pub trait Event: Message<Response = ()> + Send + Sync + 'static {
    fn event_label() -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl<T> Event for T where T: Message<Response = ()> + Send + Sync + 'static {}

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

#[derive(Debug)]
pub enum StreamEvent {
    ActorRegistered(SmallStr, ActorId),
    SubscriptionAdded(SmallStr, ActorId, SubscriptionId),
}

impl Message for StreamEvent {
    type Response = ();
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
struct StreamState {
    started: Arc<AtomicBool>,
    pending: Arc<Mutex<Vec<PendingEvent>>>,
}

struct PendingEvent {
    type_id: TypeId,
    payload: Payload,
}
#[derive(Clone, Default)]
pub struct EventStream {
    executor: Arc<Executor>,
    subscribers: Arc<RwLock<SubscriberMap>>,
    state: StreamState,
}

impl EventStream {
    pub fn new(executor: Arc<Executor>) -> Self {
        EventStream {
            executor,
            subscribers: Arc::default(),
            state: StreamState::default(),
        }
    }

    pub fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
    }

    pub fn spawn<A: Actor>(&self, actor: A) -> Addr<A> {
        let addr = Addr::spawn_with_bus(actor, Arc::clone(&self.executor), Some(self.clone()));
        self.publish(StreamEvent::ActorRegistered(addr.name().into(), addr.id));
        addr
    }

    pub fn start(&self) {
        if !self.state.started.swap(true, Ordering::AcqRel) {
            let pending = std::mem::take(&mut *self.state.pending.lock().unwrap());
            for event in pending {
                self.dispatch(event.type_id, event.payload);
            }
        }
    }

    #[inline]
    pub fn publish<E: Event>(&self, message: E) {
        if !self.state.started.load(Ordering::Acquire) {
            self.state.pending.lock().unwrap().push(PendingEvent {
                type_id: TypeId::of::<E>(),
                payload: Arc::new(message),
            });

            return;
        }

        self.dispatch(TypeId::of::<E>(), Arc::new(message));
    }

    #[inline]
    pub fn lazy_publish<E: Event>(&self, f: impl FnOnce() -> E) {
        if self.handler_count::<E>() > 0 {
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
        A: MessageHandler<E>,
    {
        let active = Arc::new(AtomicBool::new(true));
        let id = SubscriptionId::new();
        let inner_addr = addr.clone();

        let forward = Arc::new(move |payload: Payload| {
            if let Ok(msg) = payload.downcast::<E>() {
                inner_addr.broadcast(msg);
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

        self.publish(StreamEvent::SubscriptionAdded(
            addr.name().into(),
            addr.id,
            id,
        ));

        Subscription { id, active }
    }

    fn dispatch(&self, type_id: TypeId, payload: Payload) {
        let group = {
            let subscribers = self.subscribers.read().unwrap();

            match subscribers.get(&type_id) {
                Some(group) => Arc::clone(group),
                None => return,
            }
        };

        for registration in group.iter() {
            if registration.active.load(Ordering::Acquire) {
                (registration.forward)(Arc::clone(&payload));
            }
        }
    }

    fn register(&self, type_id: TypeId, registration: Registration) {
        let mut subscribers = self.subscribers.write().unwrap();

        let list = subscribers.entry(type_id).or_default();

        let registrations = Arc::make_mut(list);
        registrations.retain(|registration| registration.active.load(Ordering::Acquire));
        registrations.push(registration);
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
    fn name(&self) -> &str {
        "EventForwardActor"
    }
}

impl<M, H> MessageHandler<M> for EventForwardActor<M, H>
where
    M: Event + Message<Response = ()>,
    H: EventHandler<M>,
{
    fn handle(&mut self, msg: &M, _: &ActorContext<Self>) {
        (self.handler).handle(msg);
    }
}
