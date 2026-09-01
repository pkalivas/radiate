use crate::{
    EventHandler,
    events::{Event, Handler, Subscriber, Subscription, SubscriptionId},
};
use radiate_core::Executor;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

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

    pub fn attatch<H: EventHandler>(&self, handler: H) -> Subscriber<H> {
        let subscriber = Subscriber::new(handler, Arc::clone(&self.executor), self.clone());
        subscriber.start();
        subscriber
    }

    pub fn subscribe<E: Event>(&self, handler: impl Handler<E>) -> Subscription {
        let subscriber = Subscriber::new(handler, Arc::clone(&self.executor), self.clone());
        self.subscribe_existing::<E, _>(&subscriber)
    }

    #[inline]
    pub fn publish<E: Event>(&self, event: E) {
        let type_id = TypeId::of::<E>();
        let Some(group) = self.subscribers.read().unwrap().get(&type_id).cloned() else {
            return;
        };

        self.dispatch(&group, Arc::new(event), false);
    }

    pub fn lazy_publish<E: Event>(&self, f: impl FnOnce() -> E) {
        let type_id = TypeId::of::<E>();
        let Some(group) = self.subscribers.read().unwrap().get(&type_id).cloned() else {
            return;
        };

        let any_due = group
            .iter()
            .any(|registration| registration.subscription.reserve());

        if !any_due {
            return;
        }

        self.dispatch(&group, Arc::new(f()), true);
    }

    pub fn unsubscribe(&self, id: SubscriptionId) {
        let mut subscribers = self.subscribers.write().unwrap();
        for group in subscribers.values_mut() {
            Arc::make_mut(group).retain(|registration| registration.subscription.id() != id);
        }
    }

    pub(super) fn subscribe_existing<E, H>(&self, subscriber: &Subscriber<H>) -> Subscription
    where
        E: Event,
        H: Handler<E>,
    {
        let target = subscriber.clone();
        let forward: Forward = Arc::new(move |payload: Payload| {
            if let Ok(event) = payload.downcast::<E>() {
                target.send_shared(event);
            }
        });

        self.register::<E>(forward)
    }

    fn register<E: Event>(&self, forward: Forward) -> Subscription {
        let subscription = Subscription::new();
        let registration = Registration {
            forward,
            subscription: subscription.clone(),
        };

        let mut subscribers = self.subscribers.write().unwrap();
        let type_id = TypeId::of::<E>();
        let list = subscribers
            .entry(type_id)
            .or_insert_with(|| Arc::new(Vec::new()));
        let list = Arc::make_mut(list);

        list.retain(|registration| registration.subscription.is_alive());
        list.push(registration);

        subscription
    }

    #[inline]
    fn dispatch(&self, group: &SubscriberList, payload: Payload, scheduled: bool) {
        for registration in group.iter() {
            if !registration.subscription.is_alive() {
                continue;
            }

            if scheduled && !registration.subscription.take_permit() {
                continue;
            }

            (registration.forward)(Arc::clone(&payload));
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
