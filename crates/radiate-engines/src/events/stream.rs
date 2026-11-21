use crate::{
    EventHandler, EventScope, StreamContext, StreamStats, SubscriptionInfo,
    events::{
        Event, Handler, Subscription, SubscriptionId,
        event::Payload,
        mailbox::{MailBox, Route},
        messages::{DeferredEvent, StreamEvent},
        subscription::Counters,
    },
};
use radiate_core::{Executor, SmallStr, WaitGroup, WaitGuard, error::RadiateResult};
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::{
        Arc, RwLock, Weak,
        atomic::{AtomicUsize, Ordering},
    },
};

type SubscriberList = Arc<Vec<Registration>>;
type SubscriberMap = HashMap<TypeId, SubscriberList>;

enum DispatchMode {
    Scheduled,
    Immediate,
}

#[derive(Clone)]
struct Registration {
    route: Arc<dyn Route>,
    subscription: Subscription,
    event: &'static str,
    handler: TypeId,
}

impl Registration {
    fn subscription_info(&self) -> SubscriptionInfo {
        SubscriptionInfo {
            id: self.subscription.id(),
            mailbox_id: self.subscription.mailbox_id(),
            name: self.subscription.name(),
            event: SmallStr::from_static(self.event),
            stats: self.subscription.stats(),
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

    pub(super) fn event<E: Event>(&self) -> Option<&E> {
        self.payload.downcast_ref()
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
    context: StreamContext,
    event_count: AtomicUsize,
}

impl Shared {
    fn group<E: Event>(&self) -> Option<SubscriberList> {
        let type_id = TypeId::of::<E>();
        self.subscribers.read().unwrap().get(&type_id).cloned()
    }

    fn group_by_type(&self, id: TypeId) -> Option<SubscriberList> {
        self.subscribers.read().unwrap().get(&id).cloned()
    }

    fn scope(&self) -> EventScope {
        self.context.weak()
    }

    fn scoped(&self, guard: WaitGuard) -> EventScope {
        self.context.guarded(guard)
    }
}

// stream.rs
pub struct EventStreamScope<'a> {
    stream: &'a EventStream,
    wg: WaitGroup,
}

impl EventStreamScope<'_> {
    pub fn publish<E: Event>(&self, event: E) {
        self.stream
            .publish_scoped(event, self.stream.gaurded_scope(self.wg.guard()));
    }

    pub fn lazy_publish<E: Event>(&self, f: impl FnOnce() -> E) -> RadiateResult<()> {
        let ctx = self.stream.gaurded_scope(self.wg.guard());
        self.stream.lazy_publish_scoped(f, ctx)
    }

    pub fn publish_payload(&self, payload: Payload) {
        self.stream
            .publish_payload_scoped(payload, self.stream.gaurded_scope(self.wg.guard()));
    }

    pub fn mailboxes(&self) -> Vec<Arc<dyn Route>> {
        self.stream.mailboxes()
    }

    pub fn deferred(&self) -> Vec<Payload> {
        self.stream.deferred()
    }

    pub fn wait(&self) {
        self.wg.wait();
    }

    pub fn guard(&self) -> WaitGuard {
        self.wg.guard()
    }
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

        subscribers
            .get(&type_id)
            .map(|group| {
                group
                    .iter()
                    .filter(|reg| reg.subscription.is_alive())
                    .map(|reg| reg.subscription_info())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    /// A snapshot of every live subscription on this stream.
    /// Whether a handler of type `H` currently has a live subscription on this
    /// stream. Use it to avoid attaching the same handler twice.
    ///
    /// A handler only counts once it has subscribed to at least one event.
    /// Checking and then attaching isn't atomic, so it doesn't guard against
    /// two threads attaching the same handler at the same time.
    pub fn handler_registered<H: 'static>(&self) -> bool {
        let handler = TypeId::of::<H>();
        self.shared
            .subscribers
            .read()
            .unwrap()
            .values()
            .flat_map(|group| group.iter())
            .any(|reg| reg.handler == handler && reg.subscription.is_alive())
    }

    pub fn stats(&self) -> StreamStats {
        let subscribers = self.shared.subscribers.read().unwrap();
        StreamStats {
            event_count: self.shared.event_count.load(Ordering::SeqCst),
            subscribers: subscribers
                .values()
                .flat_map(|group| group.iter())
                .filter(|reg| reg.subscription.is_alive())
                .map(|reg| reg.subscription_info())
                .collect(),
        }
    }

    pub fn event_count(&self) -> usize {
        self.shared.event_count.load(Ordering::SeqCst)
    }

    pub fn deferred(&self) -> Vec<Payload> {
        std::mem::take(&mut *self.shared.pending.write().unwrap())
    }

    /// One route per live mailbox. A mailbox with several subscriptions
    /// appears once.
    pub fn mailboxes(&self) -> Vec<Arc<dyn Route>> {
        let mut seen = HashSet::new();
        self.shared
            .subscribers
            .read()
            .unwrap()
            .values()
            .flat_map(|group| group.iter())
            .filter(|reg| reg.subscription.is_alive() && seen.insert(reg.route.id()))
            .map(|reg| Arc::clone(&reg.route))
            .collect()
    }
}

impl EventStream {
    pub fn attach<H: EventHandler>(&self, handler: H) -> RadiateResult<MailBox<H>> {
        let subscriber = MailBox::new(handler, Arc::clone(&self.executor), self);
        subscriber.start()?;
        Ok(subscriber)
    }

    pub fn subscribe<E: Event>(&self, handler: impl Handler<E>) -> Subscription {
        let subscriber = MailBox::new(handler, Arc::clone(&self.executor), self);
        self.register::<E, _>(&subscriber)
    }

    pub fn detach<H: EventHandler>(&self) -> bool {
        let handler = TypeId::of::<H>();
        let mut subscribers = self.shared.subscribers.write().unwrap();
        let mut removed_stats = Vec::new();
        for group in subscribers.values_mut() {
            Arc::make_mut(group).retain(|r| {
                if r.handler == handler && r.subscription.is_alive() {
                    removed_stats.push(r.subscription_info());
                    false
                } else {
                    true
                }
            });
        }

        for stats in removed_stats.drain(..) {
            self.defer(StreamEvent::Removed(stats));
        }

        subscribers.retain(|_, group| !group.is_empty());
        !removed_stats.is_empty()
    }

    pub fn unsubscribe(&self, id: SubscriptionId) -> bool {
        let mut subscribers = self.shared.subscribers.write().unwrap();
        let mut total_removed = Vec::new();
        for group in subscribers.values_mut() {
            let to_remove = group
                .iter()
                .filter(|reg| reg.subscription.id() == id)
                .map(|reg| reg.subscription_info())
                .collect::<Vec<_>>();

            let removed_ids = to_remove.iter().map(|info| info.id).collect::<HashSet<_>>();

            Arc::make_mut(group)
                .retain(|registration| !removed_ids.contains(&registration.subscription.id()));

            if !to_remove.is_empty() {
                total_removed.extend(to_remove);
            }
        }

        let removed = !total_removed.is_empty();

        for stats in total_removed.drain(..) {
            self.defer(StreamEvent::Removed(stats));
        }

        removed
    }
}

impl EventStream {
    pub fn scope<R>(
        &self,
        func: impl FnOnce(EventStreamScope) -> RadiateResult<R>,
    ) -> RadiateResult<R> {
        let wg = WaitGroup::new();
        let result = func(EventStreamScope {
            stream: self,
            wg: wg.clone(),
        });
        wg.wait();
        result
    }

    pub fn flush(&self) -> RadiateResult<()> {
        let deferred = self.deferred();
        if deferred.is_empty() {
            return Ok(());
        }

        self.scope(|stream| {
            for payload in deferred {
                stream.publish_payload(payload);
            }

            Ok(())
        })
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
                    route.drain(stream.guard());
                }

                Ok(())
            })?;

            if self.event_count() == before {
                return Ok(());
            }
        }
    }

    pub fn defer<E: Event>(&self, event: E) {
        self.shared
            .pending
            .write()
            .unwrap()
            .push(Arc::new(DeferredEvent(event)));
    }

    #[inline]
    pub fn publish<E: Event>(&self, event: E) {
        self.publish_scoped(event, self.shared.scope())
    }

    pub fn lazy_publish<E: Event>(&self, f: impl FnOnce() -> E) -> RadiateResult<()> {
        self.lazy_publish_scoped(f, self.shared.scope())
    }

    pub fn publish_payload(&self, payload: Payload) {
        self.publish_payload_scoped(payload, self.shared.scope());
    }

    /// Publish `event` as part of an existing chain.
    #[inline]
    pub(crate) fn publish_scoped<E: Event>(&self, event: E, scope: EventScope) {
        if let Some(group) = self.shared.group::<E>() {
            self.dispatch(&group, Arc::new(event), scope, DispatchMode::Immediate);
        }
    }

    pub(crate) fn lazy_publish_scoped<E: Event>(
        &self,
        f: impl FnOnce() -> E,
        scope: EventScope,
    ) -> RadiateResult<()> {
        let Some(group) = self.shared.group::<E>() else {
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

        self.dispatch(&group, Arc::new(f()), scope, DispatchMode::Scheduled);
        Ok(())
    }

    pub(crate) fn publish_payload_scoped(&self, payload: Payload, ctx: EventScope) {
        if let Some(group) = self.shared.group_by_type((*payload).type_id()) {
            self.dispatch(&group, payload, ctx, DispatchMode::Immediate);
        }
    }

    #[inline]
    fn dispatch(
        &self,
        group: &SubscriberList,
        payload: Payload,
        scope: EventScope,
        mode: DispatchMode,
    ) {
        for registration in group.iter() {
            let subscription = &registration.subscription;
            if !subscription.is_alive() {
                continue;
            }

            let due = match mode {
                // lazy: already reserved
                DispatchMode::Scheduled => subscription.take_permit(),
                // eager: tick now
                DispatchMode::Immediate => subscription.is_due(),
            };

            if !due {
                continue;
            }

            subscription.counters.record_delivered();
            registration.route.deliver(Delivery {
                payload: Arc::clone(&payload),
                scope: scope.clone(),
                counters: Arc::clone(&subscription.counters),
            });
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

    pub(super) fn register<E, H>(&self, mailbox: &MailBox<H>) -> Subscription
    where
        E: Event,
        H: Handler<E>,
    {
        let subscription = Subscription::new(mailbox.id());
        let registration = Registration {
            route: mailbox.route::<E>(),
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

        drop(subscribers);

        self.defer(StreamEvent::Added(subscription.clone()));

        subscription
    }

    fn gaurded_scope(&self, guard: WaitGuard) -> EventScope {
        self.shared.scoped(guard)
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
