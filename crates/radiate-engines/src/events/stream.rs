use crate::{
    EventHandler,
    events::{
        Event, Handler, Subscriber, Subscription, SubscriptionId, SubscriptionInfo,
        messages::DeferredEvent, subscriber::EventId,
    },
};
use radiate_core::{Executor, WaitGroup, WaitGuard, error::RadiateResult};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, RwLock},
};

type Payload = Arc<dyn Any + Send + Sync>;
type Forward = Arc<dyn Fn(PayloadWrapper) + Send + Sync>;

struct PayloadWrapper {
    payload: Payload,
    id: EventId,
}

impl PayloadWrapper {
    pub fn new(payload: Payload, id: EventId) -> Self {
        Self { payload, id }
    }

    pub fn type_id(&self) -> TypeId {
        (*self.payload).type_id()
    }
}

impl Clone for PayloadWrapper {
    fn clone(&self) -> Self {
        Self {
            payload: Arc::clone(&self.payload),
            id: self.id,
        }
    }
}

#[derive(Clone)]
struct Registration {
    forward: Forward,
    subscription: Subscription,
    event: &'static str,
}

type SubscriberList = Arc<Vec<Registration>>;
type SubscriberMap = HashMap<TypeId, SubscriberList>;

#[derive(Clone, Default)]
pub struct EventStream {
    executor: Arc<Executor>,
    subscribers: Arc<RwLock<SubscriberMap>>,
    pending: Arc<RwLock<Vec<PayloadWrapper>>>,
}

impl EventStream {
    pub fn new(executor: Arc<Executor>) -> Self {
        EventStream {
            executor,
            subscribers: Arc::default(),
            pending: Arc::default(),
        }
    }

    pub fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
    }

    pub fn attach<H: EventHandler>(&self, handler: H) -> RadiateResult<Subscriber<H>> {
        let subscriber = Subscriber::new(handler, Arc::clone(&self.executor), self.clone());
        subscriber.start()?;
        Ok(subscriber)
    }

    pub fn subscribe<E: Event>(&self, handler: impl Handler<E>) -> Subscription {
        let subscriber = Subscriber::new(handler, Arc::clone(&self.executor), self.clone());
        self.subscribe_existing::<E, _>(&subscriber)
    }

    pub fn defer<E: Event>(&self, event: E) {
        self.pending.write().unwrap().push(PayloadWrapper {
            payload: Arc::new(DeferredEvent(event)),
            id: EventId::new(),
        });
    }

    #[inline]
    pub fn publish<E: Event>(&self, event: E) {
        let type_id = TypeId::of::<E>();
        let Some(group) = self.subscribers.read().unwrap().get(&type_id).cloned() else {
            return;
        };

        let wrapper = PayloadWrapper::new(Arc::new(event), EventId::new());
        self.dispatch(&group, wrapper, false);
    }

    pub fn lazy_publish<E: Event>(&self, f: impl FnOnce() -> E) -> RadiateResult<()> {
        let type_id = TypeId::of::<E>();
        let Some(group) = self.subscribers.read().unwrap().get(&type_id).cloned() else {
            return Ok(());
        };

        let mut any_due = false;
        for registration in group.iter() {
            if registration.subscription.reserve().is_ok_and(|val| val) {
                any_due = true;
            }
        }

        if !any_due {
            return Ok(());
        }

        let wrapper = PayloadWrapper::new(Arc::new(f()), EventId::new());
        self.dispatch(&group, wrapper, true);
        Ok(())
    }

    pub fn flush(&self) {
        let mut pending = self.pending.write().unwrap();
        let deferred = std::mem::take(&mut *pending);
        drop(pending);

        for wrapper in deferred {
            let type_id = wrapper.type_id();
            if let Some(group) = self.subscribers.read().unwrap().get(&type_id).cloned() {
                self.dispatch(&group, wrapper, false);
            }
        }
    }

    pub fn unsubscribe(&self, id: SubscriptionId) {
        let mut subscribers = self.subscribers.write().unwrap();
        for group in subscribers.values_mut() {
            Arc::make_mut(group).retain(|registration| registration.subscription.id() != id);
        }
    }

    /// A snapshot of every live subscription on this stream.
    pub fn stats(&self) -> Vec<SubscriptionInfo> {
        let subscribers = self.subscribers.read().unwrap();
        subscribers
            .values()
            .flat_map(|group| group.iter())
            .filter(|registration| registration.subscription.is_alive())
            .map(|registration| SubscriptionInfo {
                id: registration.subscription.id(),
                name: registration.subscription.name(),
                event: registration.event,
                stats: registration.subscription.stats(),
            })
            .collect()
    }

    pub(super) fn subscribe_existing<E, H>(&self, subscriber: &Subscriber<H>) -> Subscription
    where
        E: Event,
        H: Handler<E>,
    {
        let subscription = Subscription::new();
        let counters = Arc::clone(&subscription.counters);
        let target = subscriber.clone();

        let forward: Forward = Arc::new(move |payload: PayloadWrapper| {
            if let Ok(event) = payload.payload.downcast::<E>() {
                target.send_shared(event, Arc::clone(&counters));
            }
        });

        self.register::<E>(forward, subscription)
    }

    fn register<E: Event>(&self, forward: Forward, subscription: Subscription) -> Subscription {
        let registration = Registration {
            forward,
            subscription: subscription.clone(),
            event: std::any::type_name::<E>(),
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
    fn dispatch(&self, group: &SubscriberList, payload: PayloadWrapper, scheduled: bool) {
        for registration in group.iter() {
            if !registration.subscription.is_alive() {
                continue;
            }

            if scheduled && !registration.subscription.take_permit() {
                continue;
            }

            registration.subscription.counters.record_delivered();
            (registration.forward)(payload.clone());
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
