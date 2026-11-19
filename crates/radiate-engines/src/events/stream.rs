use crate::{
    EventHandler,
    events::{
        Event, Handler, Subscriber, Subscription, SubscriptionId, SubscriptionInfo,
        messages::DeferredEvent, subscriber::Trace,
    },
};
use radiate_core::{Executor, WaitGroup, error::RadiateResult};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt::Debug,
    sync::{
        Arc, RwLock, Weak,
        atomic::{AtomicUsize, Ordering},
    },
};

#[derive(Clone, Debug)]
pub struct StreamStats {
    pub event_count: usize,
    pub subscribers: Vec<SubscriptionInfo>,
}

type Payload = Arc<dyn Any + Send + Sync>;
type Forward = Arc<dyn Fn(PayloadWrapper) + Send + Sync>;

#[derive(Clone)]
struct PayloadWrapper {
    payload: Payload,
    trace: Trace,
}

#[derive(Clone)]
struct Registration {
    forward: Forward,
    subscription: Subscription,
    event: &'static str,
    handler: TypeId,
}

type SubscriberList = Arc<Vec<Registration>>;
type SubscriberMap = HashMap<TypeId, SubscriberList>;

/// The state every clone of an [`EventStream`] shares.
///
/// Subscribers only hold a [`Weak`] reference to it. Each registration's
/// forward closure owns a clone of its [`Subscriber`], so a strong reference
/// there would form a cycle (stream -> registration -> subscriber -> stream)
/// and no handler would ever be dropped.
#[derive(Default)]
pub(super) struct Shared {
    subscribers: RwLock<SubscriberMap>,
    pending: RwLock<Vec<Payload>>,
    event_count: AtomicUsize,
}

#[derive(Clone, Default)]
pub struct EventStream {
    executor: Arc<Executor>,
    shared: Arc<Shared>,
}

impl EventStream {
    pub fn new(executor: Arc<Executor>) -> Self {
        EventStream {
            executor,
            shared: Arc::default(),
        }
    }

    pub fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
    }

    pub fn get_subsribers<E: Event>(&self) -> Vec<SubscriptionInfo> {
        let type_id = TypeId::of::<E>();
        let subscribers = self.shared.subscribers.read().unwrap();
        let group = match subscribers.get(&type_id) {
            Some(group) => group,
            None => return Vec::new(),
        };

        group
            .iter()
            .filter(|registration| registration.subscription.is_alive())
            .map(|registration| SubscriptionInfo {
                id: registration.subscription.id(),
                name: registration.subscription.name(),
                event: registration.event,
                stats: registration.subscription.stats(),
            })
            .collect()
    }

    /// A snapshot of every live subscription on this stream.
    /// Whether a handler of type `H` currently has a live subscription on this
    /// stream. Use it to avoid attaching the same handler twice.
    ///
    /// A handler only counts once it has subscribed to at least one event.
    /// Checking and then attaching isn't atomic, so it doesn't guard against
    /// two threads attaching the same handler at the same time.
    pub fn has<H: 'static>(&self) -> bool {
        let handler = TypeId::of::<H>();
        self.shared
            .subscribers
            .read()
            .unwrap()
            .values()
            .flat_map(|group| group.iter())
            .any(|registration| {
                registration.handler == handler && registration.subscription.is_alive()
            })
    }

    pub fn stats(&self) -> StreamStats {
        let subscribers = self.shared.subscribers.read().unwrap();
        let info = subscribers
            .values()
            .flat_map(|group| group.iter())
            .filter(|registration| registration.subscription.is_alive())
            .map(|registration| SubscriptionInfo {
                id: registration.subscription.id(),
                name: registration.subscription.name(),
                event: registration.event,
                stats: registration.subscription.stats(),
            })
            .collect();

        StreamStats {
            event_count: self.shared.event_count.load(Ordering::SeqCst),
            subscribers: info,
        }
    }
}

impl EventStream {
    pub fn attach<H: EventHandler>(&self, handler: H) -> RadiateResult<Subscriber<H>> {
        let subscriber = Subscriber::new(handler, Arc::clone(&self.executor), self);
        subscriber.start()?;
        Ok(subscriber)
    }

    pub fn detach<H: EventHandler>(&self) -> bool {
        let handler = TypeId::of::<H>();
        let mut subscribers = self.shared.subscribers.write().unwrap();
        let mut removed = false;
        for group in subscribers.values_mut() {
            let before = group.len();
            Arc::make_mut(group).retain(|r| r.handler != handler);
            removed |= group.len() != before;
        }
        subscribers.retain(|_, group| !group.is_empty());
        removed
    }

    pub fn subscribe<E: Event>(&self, handler: impl Handler<E>) -> Subscription {
        let subscriber = Subscriber::new(handler, Arc::clone(&self.executor), self);
        self.subscribe_existing::<E, _>(&subscriber)
    }

    pub fn unsubscribe(&self, id: SubscriptionId) {
        let mut subscribers = self.shared.subscribers.write().unwrap();
        for group in subscribers.values_mut() {
            Arc::make_mut(group).retain(|registration| registration.subscription.id() != id);
        }
    }

    #[inline]
    pub fn publish<E: Event>(&self, event: E) {
        self.publish_traced(event, Trace::new());
    }

    /// Publish `event` as part of an existing chain.
    #[inline]
    pub(crate) fn publish_traced<E: Event>(&self, event: E, trace: Trace) {
        let type_id = TypeId::of::<E>();
        let group = self
            .shared
            .subscribers
            .read()
            .unwrap()
            .get(&type_id)
            .cloned();

        let Some(group) = group else {
            return;
        };

        let wrapper = PayloadWrapper {
            payload: Arc::new(event),
            trace,
        };
        self.dispatch(&group, wrapper, false);
    }

    pub fn lazy_publish<E: Event>(&self, f: impl FnOnce() -> E) -> RadiateResult<()> {
        let type_id = TypeId::of::<E>();
        let group = self
            .shared
            .subscribers
            .read()
            .unwrap()
            .get(&type_id)
            .cloned();

        let Some(group) = group else {
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

        let wrapper = PayloadWrapper {
            payload: Arc::new(f()),
            trace: Trace::new(),
        };
        self.dispatch(&group, wrapper, true);
        Ok(())
    }

    /// Publish every deferred event, then block until each one's whole chain
    /// has been handled: the event's own handlers and, recursively, every
    /// event those handlers published through their [`EventContext`](crate::events::EventContext).
    ///
    /// On a serial executor dispatch is synchronous, so the wait returns
    /// immediately. Only events published through a handler's context join the
    /// chain; events published some other way (a captured stream clone, a
    /// spawned thread) aren't waited on.
    ///
    /// Don't call this from inside a handler running on the stream's worker
    /// pool. It would wait on work that may need that same pool to finish.
    pub fn flush(&self) {
        let deferred = std::mem::take(&mut *self.shared.pending.write().unwrap());
        if deferred.is_empty() {
            return;
        }

        let scope = WaitGroup::new();
        for payload in deferred {
            let type_id = (*payload).type_id();
            let group = self
                .shared
                .subscribers
                .read()
                .unwrap()
                .get(&type_id)
                .cloned();

            let Some(group) = group else {
                continue;
            };

            let wrapper = PayloadWrapper {
                payload,
                trace: Trace::scoped(scope.guard()),
            };
            self.dispatch(&group, wrapper, false);
        }

        scope.wait();
    }

    pub fn defer<E: Event>(&self, event: E) {
        self.shared
            .pending
            .write()
            .unwrap()
            .push(Arc::new(DeferredEvent(event)));
    }

    fn register<E: Event, H: 'static>(
        &self,
        forward: Forward,
        subscription: Subscription,
    ) -> Subscription {
        let registration = Registration {
            forward,
            subscription: subscription.clone(),
            event: std::any::type_name::<E>(),
            handler: TypeId::of::<H>(),
        };

        let mut subscribers = self.shared.subscribers.write().unwrap();
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

        self.shared.event_count.fetch_add(1, Ordering::SeqCst);
    }
}

impl EventStream {
    /// A weak handle to this stream's shared state, for subscribers to hold.
    pub(super) fn downgrade(&self) -> Weak<Shared> {
        Arc::downgrade(&self.shared)
    }

    /// Rebuild a stream from a subscriber's weak handle, if the stream is
    /// still alive.
    pub(super) fn upgrade(executor: &Arc<Executor>, shared: &Weak<Shared>) -> Option<Self> {
        shared.upgrade().map(|shared| EventStream {
            executor: Arc::clone(executor),
            shared,
        })
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
                target.send_shared(event, Arc::clone(&counters), payload.trace);
            }
        });

        self.register::<E, H>(forward, subscription)
    }
}

impl Debug for EventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EventStream(subscribers={}, executor={:?})",
            self.shared.subscribers.read().unwrap().len(),
            self.executor,
        )
    }
}

impl From<Executor> for EventStream {
    fn from(executor: Executor) -> Self {
        EventStream::new(Arc::new(executor))
    }
}
