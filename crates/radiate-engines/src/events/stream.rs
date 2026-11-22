use crate::{
    EventHandler, EventScope, StreamSnapshot, SubscriptionSnapshot,
    events::{
        Event, On, Subscription, SubscriptionId,
        event::Payload,
        mailbox::MailBox,
        messages::{
            StreamClosed, StreamEvent, StreamOpened, SubscriptionAdded, SubscriptionRemoved,
        },
        registry::{Build, Registration, Registry},
        subscription::nanos,
    },
};
use radiate_core::{Executor, WaitGroup, WaitGuard, error::RadiateResult};
use radiate_error::radiate_bail;
use std::{
    any::TypeId,
    fmt::Debug,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

const RUNNING: u64 = 1 << 63;

struct WallClock {
    origin: quanta::Instant, // set at creation, never changes
    state: AtomicU64,
}

impl WallClock {
    fn now(&self) -> u64 {
        nanos(self.origin.elapsed())
    }

    fn open(&self) {
        let total = self.state.load(Ordering::Relaxed); // closed: time so far
        if total & RUNNING == 0 {
            self.state
                .store(RUNNING | (self.now() - total), Ordering::Relaxed);
        }
    }

    fn close(&self) {
        let s = self.state.load(Ordering::Relaxed);
        if s & RUNNING != 0 {
            self.state
                .store(self.now() - (s & !RUNNING), Ordering::Relaxed);
        }
    }

    fn elapsed(&self) -> Duration {
        let s = self.state.load(Ordering::Relaxed);
        Duration::from_nanos(match s & RUNNING {
            0 => s,                           // closed: f
            _ => self.now() - (s & !RUNNING), // open: now − virtual start
        })
    }
}

/// The state every clone of an [`EventStream`] shares.
///
/// Mailboxes only hold a [`Weak`] reference to it. Each registration's
/// route owns a clone of its [`MailBox`], so a strong reference there would
/// form a cycle (stream -> registration -> mailbox -> stream) and no handler
/// would ever be dropped.
pub(super) struct Shared {
    registry: Registry,
    pending: Mutex<Vec<Payload>>,
    closed: AtomicBool,
    wall_clock: WallClock,
}

impl Default for Shared {
    fn default() -> Self {
        let origin = quanta::Instant::now();
        Shared {
            registry: Registry::default(),
            pending: Mutex::default(),
            closed: AtomicBool::new(false),
            wall_clock: WallClock {
                origin,
                state: AtomicU64::new(0),
            },
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

impl EventStream {
    pub fn subscribers<E: Event>(&self) -> Vec<SubscriptionSnapshot> {
        self.shared.registry.subscribers(TypeId::of::<E>())
    }

    pub fn handler_registered<H: 'static>(&self) -> bool {
        self.shared.registry.has_handler(TypeId::of::<H>())
    }

    pub fn snapshot(&self) -> StreamSnapshot {
        self.shared.registry.stats()
    }

    pub fn event_count(&self) -> usize {
        self.shared.registry.event_count()
    }

    pub fn deferred(&self) -> Vec<Payload> {
        std::mem::take(&mut *self.shared.pending.lock().unwrap())
    }
}

impl EventStream {
    pub fn attach<H: EventHandler>(&self, handler: H) -> RadiateResult<MailBox<H>> {
        let mailbox = MailBox::new(handler, Arc::clone(&self.executor), self);
        mailbox.start()?;
        Ok(mailbox)
    }

    pub fn subscribe<E: Event>(&self, handler: impl On<E>) -> Subscription {
        let mailbox = MailBox::new(handler, Arc::clone(&self.executor), self);
        self.register::<E, _>(&mailbox)
    }

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
        H: On<E>,
    {
        let subscription = Subscription::new(mailbox.id());
        let registration = Registration {
            route: mailbox.route::<E>(subscription.clone()),
            subscription: subscription.clone(),
            handler: TypeId::of::<H>(),
        };

        self.shared
            .registry
            .insert(TypeId::of::<E>(), std::any::type_name::<E>(), registration);

        self.defer(StreamEvent(SubscriptionAdded(subscription.clone())));
        subscription
    }

    fn remove_where(&self, remove: impl Fn(&Registration) -> bool) -> bool {
        let removed = self.shared.registry.remove_where(remove);
        let any = !removed.is_empty();
        for subscription in removed {
            self.defer(StreamEvent(SubscriptionRemoved(subscription)));
        }

        any
    }
}

// --- Publishing ---
impl EventStream {
    pub fn defer<E: Event>(&self, event: E) {
        self.shared.pending.lock().unwrap().push(Arc::new(event));
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
        self.shared
            .registry
            .publish(TypeId::of::<E>(), scope, Build::Always, || {
                Arc::new(event) as Payload
            });
        Ok(())
    }

    pub(crate) fn lazy_publish_scoped<E: Event>(
        &self,
        f: impl FnOnce() -> E,
        scope: EventScope,
    ) -> RadiateResult<()> {
        self.ensure_open()?;
        self.shared
            .registry
            .publish(TypeId::of::<E>(), scope, Build::IfDue, || {
                Arc::new(f()) as Payload
            });
        Ok(())
    }

    pub(crate) fn publish_payload_scoped(
        &self,
        payload: Payload,
        scope: EventScope,
    ) -> RadiateResult<()> {
        self.ensure_open()?;
        self.shared
            .registry
            .publish((*payload).type_id(), scope, Build::Always, || payload);
        Ok(())
    }

    #[inline]
    fn ensure_open(&self) -> RadiateResult<()> {
        if self.shared.closed.load(Ordering::Relaxed) {
            radiate_bail!("EventStream is closed, dropping event");
        }
        Ok(())
    }
}

// --- Waiting ---
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

    pub fn open(&mut self) -> RadiateResult<()> {
        self.shared.wall_clock.open();
        self.scope(|stream| {
            stream.lazy_publish(|| StreamEvent(StreamOpened(stream.snapshot())))?;
            stream.wait();
            stream.flush_deferred()
        })
    }

    pub fn close(&self) -> RadiateResult<()> {
        self.drain()?;
        self.scope(|stream| stream.lazy_publish(|| StreamEvent(StreamClosed(stream.snapshot()))))?;
        self.shared.wall_clock.close();
        self.shared.closed.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub fn drain(&self) -> RadiateResult<()> {
        loop {
            self.scope(|stream| stream.flush_deferred())?;

            let before = self.event_count();

            self.scope(|stream| {
                for route in stream.shared.registry.mailboxes() {
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

    pub fn flush_deferred(&self) -> RadiateResult<()> {
        self.deferred()
            .into_iter()
            .try_for_each(|payload| self.publish_payload(payload))?;
        self.lazy_publish(|| StreamEvent(self.snapshot()))
    }
}

// --- Internal ---
impl EventStream {
    pub(super) fn downgrade(&self) -> Weak<Shared> {
        Arc::downgrade(&self.shared)
    }

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
        self.wg.as_ref().map(WaitGroup::wait)
    }
}

impl Debug for EventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EventStream(events={}, executor={:?})",
            self.shared.registry.event_types(),
            self.executor,
        )
    }
}

impl From<Executor> for EventStream {
    fn from(executor: Executor) -> Self {
        EventStream::new(Arc::new(executor))
    }
}
