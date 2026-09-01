use crate::{
    Actor, SmallStr, Subscription,
    events::{
        Schedule, SubscriptionId,
        addr::{ActorId, Addr, Message, MessageHandler},
    },
};
use radiate_core::Executor;
use radiate_utils::sentry_id;
use std::sync::{Mutex, atomic::AtomicUsize};
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

#[derive(Clone)]
struct PendingEvent {
    type_id: TypeId,
    payload: Payload,
}

impl Message for PendingEvent {
    type Response = ();
}

#[derive(Debug)]
pub enum StreamEvent {
    HandlerRegistered(SmallStr, ActorId),
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
        self.publish(StreamEvent::HandlerRegistered(addr.name().into(), addr.id));
        addr
    }

    pub fn register<A: Actor>(&self, actor: A) {
        let addr = Addr::from((actor, Arc::clone(&self.executor), self.clone()));
        self.publish(StreamEvent::HandlerRegistered(addr.name().into(), addr.id));
    }

    #[inline]
    pub fn publish<E: Event>(&self, message: E) {
        self.deliver(TypeId::of::<E>(), Arc::new(message));
    }

    #[inline]
    pub fn lazy_publish<E: Event>(&self, f: impl FnOnce() -> E) {
        let type_id = TypeId::of::<E>();

        let Some(group) = self.subscribers.read().unwrap().get(&type_id).cloned() else {
            return;
        };

        let any_due = group
            .iter()
            .fold(false, |any, r| r.subscription.reserve() || any);

        if !any_due {
            return;
        }

        self.deliver_to(group, Arc::new(f()), true);
    }

    pub fn unsubscribe(&self, id: SubscriptionId) {
        let mut subscribers = self.subscribers.write().unwrap();
        for group in subscribers.values_mut() {
            let mut_group = Arc::make_mut(group);
            mut_group.retain(|registration| registration.subscription.id != id);
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

        let subscription = self.subscribe_common::<E>(forward);
        self.publish(StreamEvent::FnHandler(subscription.id));
        subscription
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

    fn deliver(&self, type_id: TypeId, payload: Payload) {
        let started = self.state.started.load(Ordering::Acquire);

        if started {
            let group = {
                let subscribers = self.subscribers.read().unwrap();
                match subscribers.get(&type_id) {
                    Some(group) => Arc::clone(group),
                    None => return,
                }
            };
            self.dispatch(group, payload, false);
            return;
        }

        self.state
            .pending
            .lock()
            .unwrap()
            .push(PendingEvent { type_id, payload });

        let is_gate_event = self
            .state
            .event_gate
            .as_ref()
            .is_some_and(|gate| *gate.lock().unwrap() == type_id);

        if is_gate_event {
            self.flush_pending();
        }
    }

    fn deliver_to(&self, group: SubscriberList, payload: Payload, scheduled: bool) {
        if self.state.started.load(Ordering::Acquire) {
            self.dispatch(group, payload, scheduled);
            return;
        }
    }

    fn dispatch(&self, group: SubscriberList, payload: Payload, scheduled: bool) {
        for registration in group.iter() {
            if !registration.subscription.is_active() {
                continue;
            }

            if scheduled && !registration.subscription.take_permit() {
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
        let permits = Arc::new(AtomicUsize::new(0));
        let type_id = TypeId::of::<E>();
        let id = SubscriptionId::new();

        let registration = Registration {
            subscription: Subscription {
                id,
                active: Arc::clone(&active),
                schedule: Arc::new(RwLock::new(Schedule::default())),
                permits: Arc::clone(&permits),
            },
            forward,
        };

        let mut subscribers = self.subscribers.write().unwrap();

        let list = subscribers.entry(type_id).or_default();
        let registrations = Arc::make_mut(list);

        registrations.retain(|registration| registration.subscription.is_active());
        registrations.push(registration.clone());

        registration.subscription.clone()
    }

    fn flush_pending(&self) {
        if !self.state.started.swap(true, Ordering::AcqRel) {
            let pending = std::mem::take(&mut *self.state.pending.lock().unwrap());
            for event in pending {
                let group = {
                    let subscribers = self.subscribers.read().unwrap();
                    match subscribers.get(&event.type_id) {
                        Some(group) => Arc::clone(group),
                        None => continue,
                    }
                };
                self.dispatch(group, event.payload, false);
            }
        }
    }
}

impl Debug for EventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EventStream(subscribers={}, executor={:?})",
            self.subscribers.read().unwrap().len(),
            self.executor,
        )
    }
}

impl From<Executor> for EventStream {
    fn from(executor: Executor) -> Self {
        EventStream::new(Arc::new(executor))
    }
}
