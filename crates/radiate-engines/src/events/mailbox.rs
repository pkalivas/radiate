use crate::{
    Event, EventHandler, EventId, EventScope, On,
    events::{EventStream, Subscription, SubscriptionId, event::Payload, stream::Shared},
};
use quanta::Instant;
use radiate_core::{Executor, Inbox, WaitGuard, error::RadiateResult};
use radiate_utils::{generate_metric_key, sentry_id};
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;

sentry_id!(MailBoxId);

const PROCESS_BATCH_SIZE: usize = 64;

type HandleFn<H> = fn(&mut H, &Payload, &EventContext<'_, H>);

fn handle<E, H>(handler: &mut H, payload: &Payload, ctx: &EventContext<'_, H>)
where
    E: Event,
    H: On<E>,
{
    if let Some(event) = payload.downcast_ref::<E>() {
        handler.handle(event, ctx);
    }
}

pub trait Route: Send + Sync {
    fn id(&self) -> MailBoxId;

    fn deliver(&self, event: &Published<'_>);

    /// Queue `guard` behind everything already in the mailbox. It's released
    /// once all of that has been handled (or discarded after a panic).
    fn drain(&self, guard: WaitGuard);
}

/// One published event as the stream hands it to each route: borrowed, so a
/// mailbox that handles it inline never clones anything.
pub struct Published<'a> {
    pub(super) payload: &'a Payload,
    pub(super) scope: &'a EventScope,
    pub(super) published_at: Instant,
}

impl Published<'_> {
    /// An owned copy for a mailbox that has to queue it.
    fn to_delivery(&self, subscription: &Subscription) -> Delivery {
        Delivery {
            payload: Arc::clone(self.payload),
            scope: self.scope.clone(),
            subscription: subscription.clone(),
            published_at: self.published_at,
            finished: false,
        }
    }
}

/// A [`Route`] for event `E` into a `MailBox<H>`.
struct TypedRoute<E, H> {
    mailbox: MailBox<H>,
    subscription: Subscription,
    _event: PhantomData<fn(E)>,
}

impl<E, H> Route for TypedRoute<E, H>
where
    E: Event,
    H: On<E>,
{
    fn id(&self) -> MailBoxId {
        self.mailbox.0.id
    }

    #[inline]
    fn deliver(&self, event: &Published<'_>) {
        let mailbox = &self.mailbox;

        // Idle and not re-entrant: run it right here, straight from the
        // publisher's borrow. A re-entrant or contended call fails the claim
        // and queues behind whatever is already there, so FIFO holds.
        if !mailbox.0.executor.is_parallel() && mailbox.0.inbox.try_claim() {
            let unreached = Unreached(&self.subscription);
            mailbox.run_claimed(move |handler| {
                std::mem::forget(unreached);
                let ctx = EventContext {
                    scope: event.scope,
                    mailbox,
                };
                let started = Instant::now();
                let result = catch_unwind(AssertUnwindSafe(|| {
                    handle::<E, H>(handler, event.payload, &ctx)
                }));
                match result {
                    Ok(()) => self.subscription.record_handled(
                        event.published_at,
                        started,
                        Instant::now(),
                        true,
                    ),
                    Err(_) => self.subscription.record_panicked(),
                }
                result
            });
            return;
        }

        mailbox.push(Item::Deliver {
            delivery: event.to_delivery(&self.subscription),
            handle: handle::<E, H>,
        });
    }

    fn drain(&self, guard: WaitGuard) {
        self.mailbox.enqueue(Item::Drain(guard));
    }
}

/// Counts an inline delivery as discarded if it never reaches the handler.
/// It moves into the closure that runs the handler and is forgotten there;
/// if that closure is dropped unrun (a handler poisoned by an earlier panic
/// refuses it), dropping this records the outcome instead.
struct Unreached<'a>(&'a Subscription);

impl Drop for Unreached<'_> {
    fn drop(&mut self) {
        self.0.record_discarded();
    }
}

struct Delivery {
    payload: Payload,
    scope: EventScope,
    subscription: Subscription,
    published_at: Instant,
    finished: bool,
}

impl Delivery {
    fn scope(&self) -> &EventScope {
        &self.scope
    }

    fn payload(&self) -> &Payload {
        &self.payload
    }

    fn is_live(&self) -> bool {
        self.subscription.is_alive()
    }

    fn complete(mut self, started: Instant, ended: Instant) {
        self.finished = true;
        self.subscription
            .record_handled(self.published_at, started, ended, false);
    }

    fn panicked(mut self) {
        self.finished = true;
        self.subscription.record_panicked();
    }
}

impl Drop for Delivery {
    fn drop(&mut self) {
        if !self.finished {
            self.subscription.record_discarded();
        }
    }
}

/// One unit of work in a mailbox's queue.
enum Item<H> {
    Deliver {
        delivery: Delivery,
        handle: HandleFn<H>,
    },
    Drain(WaitGuard),
}

struct Inner<H> {
    id: MailBoxId,
    handler: Mutex<H>,
    inbox: Inbox<Item<H>>,
    executor: Arc<Executor>,
    stream: Weak<Shared>,
}

/// A handler plus its queue of pending work.
///
/// Every delivery routed to the handler is pushed onto its [`Inbox`], and at
/// most one drain runs at a time, inline on a serial executor or as a single job
/// on a worker pool. So a handler sees its events in publish order, never runs
/// on two threads at once, and never parks pool workers waiting on its lock.
///
/// An event a handler publishes to itself (directly, or through a cycle of other
/// handlers) is queued and handled after the current `handle` call returns,
/// rather than re-entering it.
pub struct MailBox<H>(Arc<Inner<H>>);

impl<H> MailBox<H> {
    /// The stream this subscriber belongs to, if it's still alive.
    fn stream(&self) -> Option<EventStream> {
        EventStream::upgrade(&self.0.executor, &self.0.stream)
    }
}

impl<H: Send + 'static> MailBox<H> {
    pub(super) fn new(handler: H, executor: Arc<Executor>, stream: &EventStream) -> Self {
        MailBox(Arc::new(Inner {
            id: MailBoxId::new(),
            handler: Mutex::new(handler),
            inbox: Inbox::new(),
            executor,
            stream: stream.downgrade(),
        }))
    }

    pub fn id(&self) -> MailBoxId {
        self.0.id
    }

    /// Subscribe this handler to `E`. If the stream has already been dropped,
    /// the returned subscription is inactive.
    pub fn subscribe<E>(&self) -> Subscription
    where
        E: Event,
        H: On<E>,
    {
        match self.stream() {
            Some(stream) => stream.register::<E, H>(self),
            None => {
                let subscription = Subscription::new(self.0.id);
                subscription.unsubscribe();
                subscription
            }
        }
    }

    pub fn unsubscribe(&self, id: SubscriptionId) {
        if let Some(stream) = self.stream() {
            stream.unsubscribe(id);
        }
    }

    /// The handle the stream keeps to route `E` events (and drains) here.
    pub(super) fn route<E>(&self, subscription: Subscription) -> Arc<dyn Route>
    where
        E: Event,
        H: On<E>,
    {
        Arc::new(TypedRoute::<E, H> {
            mailbox: self.clone(),
            subscription,
            _event: PhantomData,
        })
    }

    /// Run the handler's `start`. The inbox is claimed for the duration, so any
    /// event `start` publishes to this handler (or that arrives from elsewhere
    /// once `start` has subscribed) queues up and is drained right after.
    pub(super) fn start(&self) -> RadiateResult<()>
    where
        H: EventHandler,
    {
        let claimed = self.0.inbox.try_claim();
        let result = {
            let scope = EventScope::new();
            let ctx = EventContext {
                scope: &scope,
                mailbox: self,
            };
            self.0.handler.lock().unwrap().start(&ctx)
        };

        if claimed {
            self.process_queue();
        }

        result
    }

    fn enqueue(&self, item: Item<H>) {
        if !self.0.executor.is_parallel() && self.0.inbox.try_claim() {
            self.run_claimed(|handler| self.process_item(item, handler, &mut None));
            return;
        }

        self.push(item);
    }

    fn push(&self, item: Item<H>) {
        if self.0.inbox.push(item) {
            self.schedule();
        }
    }

    /// Start a drain on the executor. On a serial executor this runs inline.
    fn schedule(&self) {
        let mailbox = self.clone();
        self.0.executor.submit(move || mailbox.process_queue());
    }

    /// With the claim held: run `first`, then drain whatever queued up behind
    /// it, all under one lock of the handler.
    fn run_claimed(&self, first: impl FnOnce(&mut H) -> thread::Result<()>) {
        let more = {
            let mut handler = self.lock();
            if let Err(payload) = first(&mut handler) {
                self.discard();
                resume_unwind(payload); // still holding the lock, so it poisons
            }
            self.process_batch(&mut handler)
        };

        if more {
            self.schedule();
        }
    }

    fn lock(&self) -> MutexGuard<'_, H> {
        match self.0.handler.lock() {
            Ok(handler) => handler,
            Err(_) => {
                self.discard();
                panic!(
                    "event handler `{}` panicked on an earlier event",
                    std::any::type_name::<H>()
                );
            }
        }
    }

    fn process_queue(&self) {
        let more = {
            let mut handler = self.lock();
            self.process_batch(&mut handler)
        };

        if more {
            self.schedule();
        }
    }

    /// Drain one budget's worth of queued items. Must hold the claim.
    fn process_batch(&self, handler: &mut H) -> bool {
        let mut panicked = None;
        // The last clock reading in this batch. One handler's end is the next
        // one's start, so each delivery costs one read instead of two. It
        // starts empty each batch so a reading never spans the gap between
        // two drains.
        let mut cursor = None;
        let result = self.0.inbox.drain(self.budget(), |item| {
            if panicked.is_some() {
                // Drop the rest of this turn's items, releasing their
                // scopes and barrier guards.
                return;
            }

            if let Err(payload) = self.process_item(item, handler, &mut cursor) {
                panicked = Some(payload);
            }
        });

        if let Some(payload) = panicked {
            if result.more {
                self.discard();
            }

            resume_unwind(payload);
        }

        result.more
    }

    /// Handle one item. `cursor` is the latest clock reading, if one was
    /// taken just before this item: a delivery uses it as its start and
    /// leaves its own end reading there for the next item.
    fn process_item(
        &self,
        item: Item<H>,
        handler: &mut H,
        cursor: &mut Option<Instant>,
    ) -> thread::Result<()> {
        match item {
            Item::Deliver { delivery, .. } if !delivery.is_live() => Ok(()),
            Item::Deliver { delivery, handle } => {
                let ctx = EventContext {
                    scope: delivery.scope(),
                    mailbox: self,
                };

                let started = cursor.take().unwrap_or_else(Instant::now);
                let result = catch_unwind(AssertUnwindSafe(|| {
                    handle(handler, delivery.payload(), &ctx)
                }));
                match result {
                    Ok(()) => {
                        let ended = Instant::now();
                        delivery.complete(started, ended);
                        *cursor = Some(ended);
                    }
                    // Leave the cursor empty: the next item reads afresh.
                    Err(_) => delivery.panicked(),
                }
                result
            }
            Item::Drain(guard) => {
                drop(guard);
                Ok(())
            }
        }
    }

    /// Drop everything queued and release the claim. Must hold the claim.
    fn discard(&self) {
        self.0.inbox.drain(usize::MAX, drop);
    }

    fn budget(&self) -> usize {
        if self.0.executor.is_parallel() {
            PROCESS_BATCH_SIZE
        } else {
            usize::MAX
        }
    }
}

impl<H> Clone for MailBox<H> {
    fn clone(&self) -> Self {
        MailBox(Arc::clone(&self.0))
    }
}

pub struct EventContext<'a, H> {
    scope: &'a EventScope,
    mailbox: &'a MailBox<H>,
}

impl<H> EventContext<'_, H> {
    /// Publish an event as part of the chain currently being handled. If that
    /// chain is being waited on (for example by [`EventStream::flush`]), the wait
    /// also covers this event and everything it causes.
    ///
    /// Does nothing if the stream has already been dropped.
    pub fn publish<E: Event>(&self, event: E) {
        if let Some(stream) = self.mailbox.stream() {
            stream.publish_scoped(event, self.scope.clone()).ok();
        }
    }

    pub fn lazy_publish<E: Event>(&self, event: impl FnOnce() -> E + 'static) -> RadiateResult<()> {
        if let Some(stream) = self.mailbox.stream() {
            stream.lazy_publish_scoped(event, self.scope.clone())?;
        }

        Ok(())
    }

    /// The id of the event that started the chain this handler is running in.
    pub fn root_id(&self) -> EventId {
        self.scope.id()
    }

    /// The id of the mailbox this handler runs in, as it appears in
    /// [`SubscriptionInfo::mailbox_id`](crate::SubscriptionInfo).
    pub fn mailbox_id(&self) -> MailBoxId {
        self.mailbox.0.id
    }

    pub fn subscribe<E>(&self) -> Subscription
    where
        E: Event,
        H: On<E>,
    {
        const HANDLER_PREFIX: &str = "handler";

        let first = generate_metric_key::<H>(HANDLER_PREFIX);
        let second = format!("{}.{}", first, E::name());
        self.mailbox.subscribe::<E>().with_name(second)
    }
}
