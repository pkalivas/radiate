use crate::{
    Actor, Subscription,
    events::{
        Schedule, StreamEvent, SubscriptionId,
        addr::{Addr, Message, MessageHandler},
    },
};
use radiate_core::Executor;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{
        Arc, RwLock,
        atomic::{AtomicBool, Ordering},
    },
};
use std::{
    collections::VecDeque,
    sync::{Mutex, atomic::AtomicUsize},
};

const MAX_PENDING: usize = 1024;
const IMMEDIATE_EVENT: bool = false;
const SCHEDULED_EVENT: bool = true;

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
    scheduled: bool,
}

impl Message for PendingEvent {
    type Response = ();
}

type Payload = Arc<dyn Any + Send + Sync>;
type Forward = Arc<dyn Fn(Payload) + Send + Sync>;

#[derive(Clone)]
struct Registration {
    forward: Forward,
    subscription: Subscription,
}

type SubscriberTypePair = (TypeId, Vec<Registration>);
type SubscriberList = Arc<SubscriberTypePair>;
type SubscriberMap = HashMap<TypeId, SubscriberList>;

#[derive(Clone, Default)]
struct StreamState {
    started: Arc<AtomicBool>,
    pending: Arc<Mutex<VecDeque<PendingEvent>>>,
    event_gate: Option<Arc<Mutex<TypeId>>>,
}

impl StreamState {
    fn new() -> Self {
        StreamState {
            started: Arc::new(AtomicBool::new(true)),
            pending: Arc::default(),
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
        self.publish(StreamEvent::HandlerRegistered(addr.name(), addr.id));
        addr
    }

    pub fn register<A: Actor>(&self, actor: A) {
        let addr = Addr::from((actor, Arc::clone(&self.executor), self.clone()));
        self.publish(StreamEvent::HandlerRegistered(addr.name(), addr.id));
    }

    #[inline]
    pub fn publish<E: Event>(&self, message: E) {
        let type_id = TypeId::of::<E>();
        let group = self.subscribers.read().unwrap().get(&type_id).cloned();

        match group {
            Some(group) => self.deliver(group, Arc::new(message), IMMEDIATE_EVENT),
            None => self.queue_pending(type_id, Arc::new(message), IMMEDIATE_EVENT),
        }
    }

    pub fn lazy_publish<E: Event>(&self, f: impl FnOnce() -> E) {
        let type_id = TypeId::of::<E>();
        let group = self.subscribers.read().unwrap().get(&type_id).cloned();

        let Some(group) = group else {
            return;
        };

        let any_due = group.1.iter().any(|r| r.subscription.reserve());

        if !any_due {
            return;
        }

        self.deliver(group, Arc::new(f()), SCHEDULED_EVENT);
    }

    pub fn unsubscribe(&self, id: SubscriptionId) {
        let mut subscribers = self.subscribers.write().unwrap();
        for group in subscribers.values_mut() {
            let mut_group = Arc::make_mut(group);
            mut_group
                .1
                .retain(|registration| registration.subscription.id != id);
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

    pub fn subscribe_addr<M, A>(&self, addr: &Addr<A>) -> Subscription
    where
        M: Message<Response = ()> + Sync,
        A: MessageHandler<M>,
    {
        let inner_addr = addr.clone();

        let forward = Arc::new(move |payload: Payload| {
            if let Ok(msg) = payload.downcast::<M>() {
                inner_addr.send_shared(msg);
            }
        });

        let subscription = self.subscribe_common::<M>(forward);

        self.publish(StreamEvent::SubscriptionAdded(
            addr.name(),
            addr.id,
            subscription.id,
        ));

        subscription
    }

    /// Shared entry point for both `publish` and `lazy_publish` once a
    /// subscriber group is resolved: dispatch immediately if started,
    /// otherwise queue and check the gate. Keeping this in one place is
    /// the whole point — `deliver` and `deliver_to` used to duplicate the
    /// gate check and only one of them had it.
    fn deliver(&self, group: SubscriberList, payload: Payload, scheduled: bool) {
        if self.state.started.load(Ordering::Acquire) {
            self.dispatch(group, payload, scheduled);
            return;
        }
        self.queue_pending(group.0, payload, scheduled);
    }

    /// Used when no subscriber group exists yet for `type_id` (nobody's
    /// subscribed) — still needs to queue and gate-check, since a
    /// subscriber for the gate event type might not exist either but the
    /// gate should still open once its `TypeId` is published.
    fn queue_pending(&self, type_id: TypeId, payload: Payload, scheduled: bool) {
        {
            let mut pending = self.state.pending.lock().unwrap();
            if pending.len() >= MAX_PENDING {
                pending.pop_front();
            }

            pending.push_back(PendingEvent {
                type_id,
                payload,
                scheduled,
            });
        }

        let is_gate_event = self
            .state
            .event_gate
            .as_ref()
            .is_some_and(|gate| *gate.lock().unwrap() == type_id);

        if is_gate_event {
            self.flush_pending();
        }
    }

    #[inline]
    fn dispatch(&self, group: SubscriberList, payload: Payload, scheduled: bool) {
        for registration in group.1.iter() {
            if !registration.subscription.is_alive() {
                continue;
            }

            if scheduled && !registration.subscription.take_permit() {
                continue;
            }

            (registration.forward)(Arc::clone(&payload));
        }
    }

    fn subscribe_common<E: Event>(
        &self,
        forward: Arc<dyn Fn(Payload) + Send + Sync>,
    ) -> Subscription {
        let active = Arc::new(AtomicBool::new(true));
        let permits = Arc::new(AtomicUsize::new(0));
        let type_id = TypeId::of::<E>();
        let id = SubscriptionId::new();

        let registration = Registration {
            subscription: Subscription {
                id,
                alive: Arc::clone(&active),
                schedule: Arc::new(RwLock::new(Schedule::default())),
                permits: Arc::clone(&permits),
            },
            forward,
        };

        let mut subscribers = self.subscribers.write().unwrap();

        let list = subscribers
            .entry(type_id)
            .or_insert_with(|| Arc::new((type_id, Vec::new())));
        let registrations = Arc::make_mut(list);

        registrations
            .1
            .retain(|registration| registration.subscription.is_alive());
        registrations.1.push(registration.clone());

        registration.subscription.clone()
    }

    fn flush_pending(&self) {
        if !self.state.started.swap(true, Ordering::AcqRel) {
            let pending = std::mem::take(&mut *self.state.pending.lock().unwrap());
            for event in pending.into_iter() {
                let group = {
                    let subscribers = self.subscribers.read().unwrap();
                    match subscribers.get(&event.type_id) {
                        Some(group) => Arc::clone(group),
                        None => continue,
                    }
                };

                self.dispatch(group, event.payload, event.scheduled);
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
