use crate::{
    Actor, SmallStr, Subscription,
    events::{
        Schedule, SubscriptionId,
        addr::{ActorId, Addr, Message, MessageHandler},
    },
};
use radiate_core::Executor;
use radiate_utils::sentry_id;
use std::sync::Mutex;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};

sentry_id!(EventId);

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
    FnHandler(SubscriptionId),
}

impl Message for StreamEvent {
    type Response = ();
}

type Payload = Arc<dyn Any + Send + Sync>;
type Forward = Arc<dyn Fn(Payload) + Send + Sync>;

#[derive(Clone)]
struct Registration {
    forward: Forward,
    subscription: Subscription,
    active: Arc<AtomicBool>,
}

type SubscriberList = Arc<Vec<Registration>>;
type SubscriberMap = HashMap<TypeId, SubscriberList>;

#[derive(Clone, Default)]
struct StreamState {
    started: Arc<AtomicBool>,
    pending: Arc<Mutex<Vec<PendingEvent>>>,
    event_gate: Option<Arc<Mutex<TypeId>>>,
}

impl StreamState {
    fn new() -> Self {
        StreamState {
            started: Arc::new(AtomicBool::new(true)),
            pending: Arc::new(Mutex::new(Vec::new())),
            event_gate: None,
        }
    }
}

#[derive(Clone)]
struct PendingEvent {
    type_id: TypeId,
    payload: Payload,
}

impl Message for PendingEvent {
    type Response = ();
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
            state: StreamState::new(),
        }
    }

    pub fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
    }

    pub fn defer_until<E: Event>(mut self) -> Self {
        self.state.started.store(false, Ordering::Release);
        self.state.event_gate = Some(Arc::new(Mutex::new(TypeId::of::<E>())));
        self
    }

    pub fn spawn<A: Actor>(&self, actor: A) -> Addr<A> {
        let addr = Addr::from((actor, Arc::clone(&self.executor), self.clone()));
        self.publish(StreamEvent::ActorRegistered(addr.name().into(), addr.id));
        addr
    }

    pub fn register<A: Actor>(&self, actor: A) {
        let addr = Addr::from((actor, Arc::clone(&self.executor), self.clone()));
        self.publish(StreamEvent::ActorRegistered(addr.name().into(), addr.id));
    }

    #[inline]
    pub fn publish<E: Event>(&self, message: E) {
        if !self.can_publish::<E>() {
            self.publish_internal(message);
        }
    }

    #[inline]
    pub fn lazy_publish<E: Event>(&self, f: impl FnOnce() -> E) {
        if self.can_publish::<E>() {
            self.publish_internal(f());
        }
    }

    pub fn unsubscribe(&self, id: SubscriptionId) {
        let mut subscribers = self.subscribers.write().unwrap();
        for group in subscribers.values_mut() {
            let mut_group = Arc::make_mut(group);
            mut_group.retain(|registration| {
                if registration.subscription.id == id {
                    false
                } else {
                    true
                }
            });
        }
    }

    pub fn subscribe<E>(&self, handler: impl EventHandler<E>) -> Subscription
    where
        E: Event,
    {
        let wrapped_handler = Arc::new(Mutex::new(handler));

        let forward = Arc::new(move |payload: Payload| {
            if let Ok(event) = payload.downcast::<E>() {
                wrapped_handler.lock().unwrap().handle(event.as_ref());
            }
        });

        self.subscribe_common::<E>(forward)
    }

    pub fn subscribe_addr<E, A>(&self, addr: &Addr<A>) -> Subscription
    where
        E: Event,
        A: MessageHandler<E>,
    {
        let inner_addr = addr.clone();

        let forward = Arc::new(move |payload: Payload| {
            if let Ok(msg) = payload.downcast::<E>() {
                inner_addr.send_shared(msg);
            }
        });

        let subscription = self.subscribe_common::<E>(forward);

        self.publish(StreamEvent::SubscriptionAdded(
            addr.name().into(),
            addr.id,
            subscription.id,
        ));

        subscription
    }

    #[inline]
    fn publish_internal<E: Event>(&self, message: E) {
        let type_id = TypeId::of::<E>();
        let started = self.state.started.load(Ordering::Acquire);

        if started {
            self.dispatch(type_id, Arc::new(message));
            return;
        }

        self.state.pending.lock().unwrap().push(PendingEvent {
            type_id,
            payload: Arc::new(message),
        });

        if let Some(event_gate) = &self.state.event_gate {
            if type_id == *event_gate.lock().unwrap() {
                self.flush_pending();
            }
        }
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
            if !registration.active.load(Ordering::Acquire) {
                continue;
            }

            (registration.forward)(Arc::clone(&payload));
        }
    }

    fn subscribe_common<E>(&self, forward: Arc<dyn Fn(Payload) + Send + Sync>) -> Subscription
    where
        E: Event,
    {
        let active = Arc::new(AtomicBool::new(true));
        let type_id = TypeId::of::<E>();
        let id = SubscriptionId::new();
        let registration = Registration {
            subscription: Subscription {
                id,
                active: Arc::clone(&active),
                schedule: Arc::new(RwLock::new(Schedule::default())),
            },
            forward,
            active: Arc::clone(&active),
        };

        let mut subscribers = self.subscribers.write().unwrap();

        let list = subscribers.entry(type_id).or_default();
        let registrations = Arc::make_mut(list);

        registrations.retain(|registration| registration.active.load(Ordering::Acquire));
        registrations.push(registration.clone());

        registration.subscription.clone()
    }

    fn flush_pending(&self) {
        if !self.state.started.swap(true, Ordering::AcqRel) {
            let pending = std::mem::take(&mut *self.state.pending.lock().unwrap());
            for event in pending {
                self.dispatch(event.type_id, event.payload);
            }
        }
    }

    fn can_publish<E: Event>(&self) -> bool {
        self.subscribers
            .read()
            .unwrap()
            .get(&TypeId::of::<E>())
            .is_some_and(|group| {
                group.iter().any(|registration| {
                    registration.active.load(Ordering::Acquire)
                        && registration.subscription.try_schedule()
                })
            })
    }
}

impl Debug for EventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let subscribers = self.subscribers.read().unwrap();
        let mut registrations = String::new();

        for (type_id, group) in subscribers.iter() {
            for registration in group.iter() {
                registrations.push_str(&format!(
                    "TypeId={:?}, id={}, active={}",
                    type_id,
                    registration.subscription.id,
                    registration.active.load(Ordering::Acquire),
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

impl From<Executor> for EventStream {
    fn from(executor: Executor) -> Self {
        EventStream::new(Arc::new(executor))
    }
}
