use crate::{
    EventHandler, EventScope, StreamSnapshot, SubscriptionInfo,
    events::{
        Event, Handler, Subscription, SubscriptionId,
        event::Payload,
        mailbox::{MailBox, Route},
        messages::StreamEvent,
        subscription::Counters,
    },
};
use radiate_core::{Executor, SmallStr, WaitGroup, WaitGuard, error::RadiateResult};
use radiate_error::radiate_bail;
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::{
        Arc, RwLock, Weak,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

type SubscriberList = Arc<Vec<Registration>>;
type SubscriberMap = HashMap<TypeId, SubscriberList>;

#[derive(Clone, Debug)]
pub struct StreamOpened;

#[derive(Clone, Debug)]
pub struct StreamClosed(pub StreamSnapshot);

#[derive(Clone, Debug)]
pub struct SubscriptionAdded(pub Subscription);

#[derive(Clone, Debug)]
pub struct SubscriptionRemoved(pub Subscription);

enum DispatchMode {
    Lazy,
    Eager,
}

#[derive(Clone)]
struct Registration {
    route: Arc<dyn Route>,
    subscription: Subscription,
    event: &'static str,
    handler: TypeId,
}

impl Registration {
    fn is_alive(&self) -> bool {
        self.subscription.is_alive()
    }

    fn info(&self) -> SubscriptionInfo {
        SubscriptionInfo {
            id: self.subscription.id(),
            mailbox_id: self.subscription.mailbox_id(),
            name: self.subscription.name(),
            event: SmallStr::from_static(self.event),
            stats: self.subscription.stats(),
        }
    }
}

/// One published event as the stream hands it to each route: borrowed, so a
/// mailbox that handles it inline never clones anything.
pub struct Published<'a> {
    pub(super) payload: &'a Payload,
    pub(super) scope: &'a EventScope,
}

impl Published<'_> {
    /// An owned copy for a mailbox that has to queue it.
    pub(super) fn to_delivery(&self, counters: &Arc<Counters>) -> Delivery {
        Delivery {
            payload: Arc::clone(self.payload),
            scope: self.scope.clone(),
            counters: Arc::clone(counters),
        }
    }
}

/// One event routed to one subscription: what the stream hands a mailbox.
///
/// The mailbox reads it and calls [`complete`](Delivery::complete) once the
/// handler returns. Dropping it without completing (a panic, a discard) still
/// releases its scope, but it isn't counted as handled.
pub struct Delivery {
    payload: Payload,
    scope: EventScope,
    counters: Arc<Counters>,
}

impl Delivery {
    pub(super) fn scope(&self) -> &EventScope {
        &self.scope
    }

    pub(super) fn payload(&self) -> &Payload {
        &self.payload
    }

    /// The handler finished with this delivery.
    pub(super) fn complete(self) {
        self.counters.record_handled();
    }
}

/// The state every clone of an [`EventStream`] shares.
///
/// Mailboxes only hold a [`Weak`] reference to it. Each registration's
/// [`Route`] owns a clone of its [`MailBox`], so a strong reference there
/// would form a cycle (stream -> registration -> mailbox -> stream) and no
/// handler would ever be dropped.
#[derive(Default)]
pub(super) struct Shared {
    subscribers: RwLock<SubscriberMap>,
    pending: RwLock<Vec<Payload>>,
    event_count: AtomicUsize,
    closed: AtomicBool,
}

impl Shared {
    #[inline]
    fn group(&self, event: TypeId) -> Option<SubscriberList> {
        self.subscribers.read().unwrap().get(&event).cloned()
    }

    /// Run `f` over every live registration, under the read lock.
    fn live<R>(&self, f: impl FnOnce(&mut dyn Iterator<Item = &Registration>) -> R) -> R {
        let subscribers = self.subscribers.read().unwrap();
        let mut live = subscribers
            .values()
            .flat_map(|group| group.iter())
            .filter(|reg| reg.is_alive());
        f(&mut live)
    }
}

#[derive(Clone, Default)]
pub struct EventStream {
    executor: Arc<Executor>,
    shared: Arc<Shared>,
    wg: Option<WaitGroup>,
}

impl EventStream {
    pub fn new(executor: Arc<Executor>) -> Self {
        EventStream {
            executor,
            shared: Arc::default(),
            wg: None,
        }
    }

    pub fn set_executor(&mut self, executor: Arc<Executor>) {
        self.executor = executor;
    }
}

/// Queries
impl EventStream {
    /// The live subscriptions to `E`.
    pub fn subscribers<E: Event>(&self) -> Vec<SubscriptionInfo> {
        self.shared
            .group(TypeId::of::<E>())
            .map(|group| {
                group
                    .iter()
                    .filter(|reg| reg.is_alive())
                    .map(Registration::info)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Whether a handler of type `H` currently has a live subscription on this
    /// stream. Use it to avoid attaching the same handler twice.
    ///
    /// A handler only counts once it has subscribed to at least one event.
    /// Checking and then attaching isn't atomic, so it doesn't guard against
    /// two threads attaching the same handler at the same time.
    pub fn handler_registered<H: 'static>(&self) -> bool {
        let handler = TypeId::of::<H>();
        self.shared
            .live(|live| live.map(|reg| reg.handler).any(|h| h == handler))
    }

    /// A snapshot of every live subscription on this stream.
    pub fn stats(&self) -> StreamSnapshot {
        StreamSnapshot {
            event_count: self.event_count(),
            subscribers: self
                .shared
                .live(|live| live.map(Registration::info).collect()),
        }
    }

    pub fn event_count(&self) -> usize {
        self.shared.event_count.load(Ordering::SeqCst)
    }

    /// Take every deferred event, leaving none behind.
    pub fn deferred(&self) -> Vec<Payload> {
        std::mem::take(&mut *self.shared.pending.write().unwrap())
    }

    /// One route per live mailbox. A mailbox with several subscriptions
    /// appears once.
    pub fn mailboxes(&self) -> Vec<Arc<dyn Route>> {
        let mut seen = HashSet::new();
        self.shared.live(|live| {
            live.filter(|reg| seen.insert(reg.route.id()))
                .map(|reg| Arc::clone(&reg.route))
                .collect()
        })
    }
}

/// Subscribing
impl EventStream {
    pub fn attach<H: EventHandler>(&self, handler: H) -> RadiateResult<MailBox<H>> {
        let mailbox = MailBox::new(handler, Arc::clone(&self.executor), self);
        mailbox.start()?;
        Ok(mailbox)
    }

    pub fn subscribe<E: Event>(&self, handler: impl Handler<E>) -> Subscription {
        let mailbox = MailBox::new(handler, Arc::clone(&self.executor), self);
        self.register::<E, _>(&mailbox)
    }

    /// Remove every live subscription held by a handler of type `H`.
    pub fn detach<H: EventHandler>(&self) -> bool {
        let handler = TypeId::of::<H>();
        self.remove_where(|reg| reg.handler == handler && reg.is_alive())
    }

    pub fn unsubscribe(&self, id: SubscriptionId) -> bool {
        self.remove_where(|reg| reg.subscription.id() == id)
    }

    pub(super) fn register<E, H>(&self, mailbox: &MailBox<H>) -> Subscription
    where
        E: Event,
        H: Handler<E>,
    {
        let subscription = Subscription::new(mailbox.id());
        let registration = Registration {
            route: mailbox.route::<E>(Arc::clone(subscription.counters())),
            subscription: subscription.clone(),
            event: std::any::type_name::<E>(),
            handler: TypeId::of::<H>(),
        };

        {
            let mut subscribers = self.shared.subscribers.write().unwrap();
            let group = Arc::make_mut(subscribers.entry(TypeId::of::<E>()).or_default());
            group.retain(Registration::is_alive);
            group.push(registration);
        }

        self.defer(StreamEvent(SubscriptionAdded(subscription.clone())));
        subscription
    }

    /// Drop every registration matching `remove`, deferring a
    /// [`StreamEvent::Removed`] for each.
    fn remove_where(&self, remove: impl Fn(&Registration) -> bool) -> bool {
        let mut removed = Vec::new();
        {
            let mut subscribers = self.shared.subscribers.write().unwrap();
            for group in subscribers.values_mut() {
                Arc::make_mut(group).retain(|reg| {
                    if remove(reg) {
                        removed.push(reg.subscription.clone());
                        false
                    } else {
                        true
                    }
                });
            }
            subscribers.retain(|_, group| !group.is_empty());
        }

        let any = !removed.is_empty();
        for info in removed {
            self.defer(StreamEvent(SubscriptionRemoved(info)));
        }
        any
    }
}

/// Publishing
impl EventStream {
    pub fn defer<E: Event>(&self, event: E) {
        self.shared.pending.write().unwrap().push(Arc::new(event));
    }

    #[inline]
    pub fn publish<E: Event>(&self, event: E) -> RadiateResult<()> {
        self.publish_scoped(event, self.event_scope())
    }

    pub fn lazy_publish<E: Event>(&self, f: impl FnOnce() -> E) -> RadiateResult<()> {
        self.lazy_publish_scoped(f, self.event_scope())
    }

    pub fn publish_payload(&self, payload: Payload) -> RadiateResult<()> {
        self.publish_payload_scoped(payload, self.event_scope())
    }

    #[inline]
    pub(crate) fn publish_scoped<E: Event>(
        &self,
        event: E,
        scope: EventScope,
    ) -> RadiateResult<()> {
        if self.shared.closed.load(Ordering::Relaxed) {
            radiate_bail!("EventStream is closed, dropping event")
        }

        if let Some(group) = self.shared.group(TypeId::of::<E>()) {
            self.dispatch(&group, Arc::new(event), scope, DispatchMode::Eager);
        }

        Ok(())
    }

    pub(crate) fn lazy_publish_scoped<E: Event>(
        &self,
        f: impl FnOnce() -> E,
        scope: EventScope,
    ) -> RadiateResult<()> {
        if self.shared.closed.load(Ordering::Relaxed) {
            radiate_bail!("EventStream is closed, dropping event");
        }

        let Some(group) = self.shared.group(TypeId::of::<E>()) else {
            return Ok(());
        };

        // Not `any`: every subscription has to tick, even after one is due.
        let reserved = group
            .iter()
            .fold(false, |any, reg| reg.subscription.reserve() | any);

        if reserved {
            self.dispatch(&group, Arc::new(f()), scope, DispatchMode::Lazy);
        }

        Ok(())
    }

    pub(crate) fn publish_payload_scoped(
        &self,
        payload: Payload,
        scope: EventScope,
    ) -> RadiateResult<()> {
        if self.shared.closed.load(Ordering::Relaxed) {
            radiate_bail!("EventStream is closed, dropping event");
        }

        if let Some(group) = self.shared.group((*payload).type_id()) {
            self.dispatch(&group, payload, scope, DispatchMode::Eager);
        }

        Ok(())
    }

    #[inline]
    fn dispatch(
        &self,
        group: &SubscriberList,
        payload: Payload,
        scope: EventScope,
        mode: DispatchMode,
    ) {
        let published = Published {
            payload: &payload,
            scope: &scope,
        };

        for registration in group.iter() {
            let subscription = &registration.subscription;
            if !subscription.is_alive() {
                continue;
            }

            let due = match mode {
                DispatchMode::Lazy => subscription.take_reserved(),
                DispatchMode::Eager => subscription.tick(),
            };

            if due {
                subscription.counters().record_delivered();
                registration.route.deliver(&published);
            }
        }

        // Relaxed: drain reads this after a WaitGroup handoff, which orders it.
        self.shared.event_count.fetch_add(1, Ordering::Relaxed);
    }
}

/// Waiting
impl EventStream {
    pub fn scope<R>(&self, func: impl FnOnce(EventStream) -> RadiateResult<R>) -> RadiateResult<R> {
        let wg = WaitGroup::new();
        let result = func(EventStream {
            executor: self.executor.clone(),
            shared: self.shared.clone(),
            wg: Some(wg.clone()),
        });
        wg.wait();
        result
    }

    pub fn open(&self) -> RadiateResult<()> {
        self.scope(|stream| {
            stream.lazy_publish(|| StreamEvent(StreamOpened))?;
            stream.wait();
            self.deferred()
                .into_iter()
                .map(|payload| stream.publish_payload(payload))
                .collect::<RadiateResult<()>>()
        })
    }

    pub fn close(&self) -> RadiateResult<()> {
        loop {
            self.flush()?;
            let before = self.event_count();

            self.scope(|stream| {
                for route in stream.mailboxes() {
                    match stream.guard() {
                        Some(guard) => route.drain(guard),
                        None => radiate_bail!(Engine: "Failed to acquire wait guard"),
                    }
                }

                Ok(())
            })?;

            if self.event_count() == before {
                break;
            }
        }

        self.scope(|stream| stream.lazy_publish(|| StreamEvent(StreamClosed(stream.stats()))))?;
        self.shared.closed.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn snapshot(&self) -> RadiateResult<()> {
        self.lazy_publish(|| StreamEvent(self.stats()))
    }

    /// Block until every mailbox has handled everything published so far,
    /// including events handlers publish while catching up.
    ///
    /// Each pass queues a barrier behind the work already in every mailbox and
    /// waits for all of them to pass it. If handlers published more in the
    /// meantime, it goes around again.
    ///
    /// Don't call this from inside a handler: its own mailbox can't pass the
    /// barrier until the handler returns.
    pub fn drain(&self) -> RadiateResult<()> {
        loop {
            self.flush()?;
            let before = self.event_count();

            self.scope(|stream| {
                for route in stream.mailboxes() {
                    match stream.guard() {
                        Some(guard) => route.drain(guard),
                        None => radiate_bail!(Engine: "Failed to acquire wait guard"),
                    }
                }

                Ok(())
            })?;

            if self.event_count() == before {
                // self.scope(|stream| stream.lazy_publish(|| StreamSnapshot(stream.stream.stats())))?;
                return Ok(());
            }
        }
    }

    /// Publish everything deferred so far and wait for it to be handled,
    /// along with everything it causes.
    pub fn flush(&self) -> RadiateResult<()> {
        let deferred = self.deferred();
        if deferred.is_empty() {
            return Ok(());
        }

        self.scope(|stream| {
            for payload in deferred {
                stream.publish_payload(payload)?;
            }

            Ok(())
        })
    }
}

/// Internal
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
            wg: None,
        })
    }

    pub(super) fn guard(&self) -> Option<WaitGuard> {
        self.wg.as_ref().map(|wg| wg.guard())
    }

    pub(super) fn event_scope(&self) -> EventScope {
        match self.guard() {
            Some(guard) => EventScope::guarded(guard),
            None => EventScope::new(),
        }
    }

    pub(super) fn wait(&self) -> Option<usize> {
        match self.wg.as_ref() {
            Some(wg) => Some(wg.wait()),
            None => return None,
        }
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
