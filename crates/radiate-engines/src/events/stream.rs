use crate::{
    EventHandler, EventScope, EventStats, StreamSnapshot, SubscriptionInfo,
    events::{
        Event, Handler, Subscription, SubscriptionId,
        event::Payload,
        mailbox::{MailBox, Route},
        messages::StreamEvent,
        subscription::Counters,
    },
};
use arc_swap::ArcSwap;
use radiate_core::{Executor, SmallStr, WaitGroup, WaitGuard, error::RadiateResult};
use radiate_error::radiate_bail;
use smallvec::SmallVec;
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::{
        Arc, Mutex, PoisonError, RwLock, Weak,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Instant,
};

use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default)]
struct TypeIdHasher(u64);

impl Hasher for TypeIdHasher {
    // The path TypeId takes today: keep the value as the hash.
    fn write_u64(&mut self, n: u64) {
        self.0 = n;
    }

    // Fallback in case std ever hashes TypeId as bytes. That's an
    // implementation detail, not a guarantee, so stay correct if it changes.
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0.rotate_left(5) ^ b as u64).wrapping_mul(0x517c_c1b7_2722_0a95);
        }
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

type SubscriberMap = HashMap<TypeId, SubscriberList, BuildHasherDefault<TypeIdHasher>>;

type SubscriberList = Arc<Group>;
// type SubscriberMap = HashMap<TypeId, SubscriberList>;
type Due<'a> = SmallVec<[&'a Registration; 4]>;

#[derive(Clone, Debug)]
pub struct StreamOpened;

#[derive(Clone, Debug)]
pub struct StreamClosed(pub StreamSnapshot);

#[derive(Clone, Debug)]
pub struct SubscriptionAdded(pub Subscription);

#[derive(Clone, Debug)]
pub struct SubscriptionRemoved(pub Subscription);

/// Everything subscribed to one event type. Copied on write when a
/// subscription is added or removed; the counts are shared across copies.
#[derive(Clone)]
struct Group {
    registrations: Vec<Registration>,
    counts: Arc<EventCounts>,
}

impl Group {
    fn new(event: &'static str) -> Self {
        Group {
            registrations: Vec::new(),
            counts: Arc::new(EventCounts {
                event,
                ..Default::default()
            }),
        }
    }

    /// Tick every registration and keep the ones that are due. `collect` drives
    /// the whole iterator, so every schedule advances once per event, even
    /// after the first due one is found.
    #[inline]
    fn due(&self) -> Due<'_> {
        self.registrations
            .iter()
            .filter(|reg| reg.subscription.tick())
            .collect()
    }
}

/// Live per-event-type counters behind [`EventStats`].
#[derive(Default)]
struct EventCounts {
    event: &'static str,
    published: AtomicU64,
    unbuilt: AtomicU64,
    undelivered: AtomicU64,
}

impl EventCounts {
    fn stats(&self) -> EventStats {
        EventStats {
            event: SmallStr::from_static(self.event),
            published: self.published.load(Ordering::Relaxed),
            unbuilt: self.unbuilt.load(Ordering::Relaxed),
            undelivered: self.undelivered.load(Ordering::Relaxed),
        }
    }
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
///
/// A group is never removed once created, even when its last subscription
/// goes, so its counts (and [`EventStream::event_count`]) only ever grow.
pub(super) struct Shared {
    subscribers: ArcSwap<SubscriberMap>,
    writer: Mutex<()>,
    pending: RwLock<Vec<Payload>>,
    closed: AtomicBool,
    created: Instant,
}

impl Shared {
    fn update<R>(&self, f: impl FnOnce(&mut SubscriberMap) -> R) -> R {
        let (out, old) = {
            // The guarded value is (), so a poisoned lock protects nothing.
            let _writer = self.writer.lock().unwrap_or_else(PoisonError::into_inner);
            let mut next = SubscriberMap::clone(&self.subscribers.load());
            let out = f(&mut next);
            (out, self.subscribers.swap(Arc::new(next)))
        };

        // Outside the lock: this may be the last reference to removed handlers,
        // and their Drop is allowed to subscribe or unsubscribe.
        drop(old);
        out
    }

    fn live<R>(&self, f: impl FnOnce(&mut dyn Iterator<Item = &Registration>) -> R) -> R {
        let map = self.subscribers.load();
        let mut live = map
            .values()
            .flat_map(|group| group.registrations.iter())
            .filter(|reg| reg.is_alive());
        f(&mut live)
    }
}

impl Default for Shared {
    fn default() -> Self {
        Shared {
            subscribers: ArcSwap::from_pointee(HashMap::default()),
            writer: Mutex::default(),
            pending: RwLock::default(),
            closed: AtomicBool::new(false),
            created: Instant::now(),
        }
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
            .subscribers
            .load()
            .get(&TypeId::of::<E>())
            .map(|group| {
                group
                    .registrations
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

    /// A snapshot of every event type and live subscription on this stream.
    pub fn stats(&self) -> StreamSnapshot {
        let at = self.shared.created.elapsed();
        let map = self.shared.subscribers.load();

        let events: Vec<_> = map.values().map(|group| group.counts.stats()).collect();
        let subscribers = map
            .values()
            .flat_map(|group| group.registrations.iter())
            .filter(|reg| reg.is_alive())
            .map(Registration::info)
            .collect();

        StreamSnapshot {
            at,
            event_count: events.iter().map(|e| e.published as usize).sum(),
            events,
            subscribers,
        }
    }

    pub fn event_count(&self) -> usize {
        self.shared
            .subscribers
            .load()
            .values()
            .map(|group| group.counts.published.load(Ordering::Relaxed) as usize)
            .sum()
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

        // register
        self.shared.update(|map| {
            let group = map
                .entry(TypeId::of::<E>())
                .or_insert_with(|| Arc::new(Group::new(registration.event)));
            let registrations = &mut Arc::make_mut(group).registrations;
            registrations.retain(Registration::is_alive);
            registrations.push(registration);
        });

        self.defer(StreamEvent(SubscriptionAdded(subscription.clone())));
        subscription
    }

    /// Drop every registration matching `remove`, deferring a
    /// [`SubscriptionRemoved`] for each. Emptied groups stay, with their counts.
    fn remove_where(&self, remove: impl Fn(&Registration) -> bool) -> bool {
        let removed = self.shared.update(|map| {
            let mut removed = Vec::new();
            for group in map.values_mut() {
                // Only copy groups that actually change. make_mut always copies here,
                // because the previous map still shares every group.
                if !group.registrations.iter().any(&remove) {
                    continue;
                }
                Arc::make_mut(group).registrations.retain(|reg| {
                    if remove(reg) {
                        reg.subscription.unsubscribe();
                        removed.push(reg.subscription.clone());
                        false
                    } else {
                        true
                    }
                });
            }
            removed
        });

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
        self.ensure_open()?;

        let map = self.shared.subscribers.load();
        if let Some(group) = map.get(&TypeId::of::<E>()) {
            let due = group.due();
            self.dispatch(group, &due, Arc::new(event), scope);
        }

        Ok(())
    }

    /// Ticks every schedule, and builds the event only if at least one
    /// subscription is due. If none is, the event is counted as unbuilt and
    /// never constructed.
    pub(crate) fn lazy_publish_scoped<E: Event>(
        &self,
        f: impl FnOnce() -> E,
        scope: EventScope,
    ) -> RadiateResult<()> {
        self.ensure_open()?;

        let map = self.shared.subscribers.load();
        let Some(group) = map.get(&TypeId::of::<E>()) else {
            return Ok(());
        };

        let due = group.due();
        if due.is_empty() {
            group.counts.unbuilt.fetch_add(1, Ordering::Relaxed);
            return Ok(());
        }

        self.dispatch(group, &due, Arc::new(f()), scope);
        Ok(())
    }

    pub(crate) fn publish_payload_scoped(
        &self,
        payload: Payload,
        scope: EventScope,
    ) -> RadiateResult<()> {
        self.ensure_open()?;

        let map = self.shared.subscribers.load();
        if let Some(group) = map.get(&(*payload).type_id()) {
            let due = group.due();
            self.dispatch(group, &due, payload, scope);
        }

        Ok(())
    }

    /// Deliver to the registrations that were due when the event was
    /// published. Liveness is checked again here: building the payload, or a
    /// handler run inline earlier in this loop, may have unsubscribed one.
    #[inline]
    fn dispatch(&self, group: &Group, due: &[&Registration], payload: Payload, scope: EventScope) {
        let published = Published {
            payload: &payload,
            scope: &scope,
        };

        let mut delivered = false;
        for reg in due {
            if !reg.is_alive() {
                continue;
            }

            delivered = true;
            reg.subscription.counters().record_delivered();
            reg.route.deliver(&published);
        }

        // Relaxed: drain reads these after a WaitGroup handoff, which orders them.
        group.counts.published.fetch_add(1, Ordering::Relaxed);
        if !delivered {
            group.counts.undelivered.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[inline]
    fn ensure_open(&self) -> RadiateResult<()> {
        if self.shared.closed.load(Ordering::Relaxed) {
            radiate_bail!("EventStream is closed, dropping event");
        }
        Ok(())
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
        self.drain()?;
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
            self.scope(|stream| {
                stream
                    .deferred()
                    .into_iter()
                    .map(|payload| stream.publish_payload(payload))
                    .collect::<RadiateResult<()>>()
            })?;

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
                return Ok(());
            }
        }
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
            self.shared.subscribers.load().len(),
            self.executor,
        )
    }
}

impl From<Executor> for EventStream {
    fn from(executor: Executor) -> Self {
        EventStream::new(Arc::new(executor))
    }
}
