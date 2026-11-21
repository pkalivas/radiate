use crate::{
    Event, EventHandler, EventId, EventScope, Handler,
    events::{
        EventStream, Subscription, SubscriptionId,
        event::Payload,
        stream::{Delivery, Published, Shared},
        subscription::Counters,
    },
};
use radiate_core::{Executor, Inbox, WaitGuard, error::RadiateResult};
use radiate_utils::{generate_metric_key, sentry_id};
use std::marker::PhantomData;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::thread;

sentry_id!(MailBoxId);

/// Deliveries a [`MailBox`] drains per turn on a worker pool before handing the
/// worker back to the executor. On a serial executor the drain just keeps going.
const PROCESS_BATCH_SIZE: usize = 64;

/// The stream's only handle to a mailbox: one per registration, typed on the
/// event behind the trait object so the stream never sees the handler type.
pub trait Route: Send + Sync {
    /// The mailbox this route leads to. Several routes can share one.
    fn id(&self) -> MailBoxId;

    /// Hand an event to the mailbox: handled inline if it's idle on a serial
    /// executor, otherwise queued.
    fn deliver(&self, event: &Published<'_>);

    /// Queue `guard` behind everything already in the mailbox. It's released
    /// once all of that has been handled (or discarded after a panic).
    fn drain(&self, guard: WaitGuard);
}

/// A [`Route`] for event `E` into a `MailBox<H>`.
struct TypedRoute<E, H> {
    mailbox: MailBox<H>,
    counters: Arc<Counters>,
    _event: PhantomData<fn(E)>,
}

impl<E, H> Route for TypedRoute<E, H>
where
    E: Event,
    H: Handler<E>,
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
            mailbox.run_claimed(|handler| {
                let ctx = EventContext {
                    scope: event.scope,
                    mailbox,
                };
                catch_unwind(AssertUnwindSafe(|| {
                    handle::<E, H>(handler, event.payload, &ctx)
                }))?;
                self.counters.record_handled();
                Ok(())
            });
            return;
        }

        mailbox.push(Item::Deliver {
            delivery: event.to_delivery(&self.counters),
            handle: handle::<E, H>,
        });
    }

    fn drain(&self, guard: WaitGuard) {
        self.mailbox.enqueue(Item::Drain(guard));
    }
}

/// Monomorphized per (E, H) and stored as a fn pointer, so one queue can hold
/// deliveries of every event type the handler subscribes to.
type HandleFn<H> = fn(&mut H, &Payload, &EventContext<'_, H>);

fn handle<E, H>(handler: &mut H, payload: &Payload, ctx: &EventContext<'_, H>)
where
    E: Event,
    H: Handler<E>,
{
    if let Some(event) = payload.downcast_ref::<E>() {
        handler.handle(event, ctx);
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
        H: Handler<E>,
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
    pub(super) fn route<E>(&self, counters: Arc<Counters>) -> Arc<dyn Route>
    where
        E: Event,
        H: Handler<E>,
    {
        Arc::new(TypedRoute::<E, H> {
            mailbox: self.clone(),
            counters,
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
            self.run_claimed(|handler| self.process_item(item, handler));
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
        let more = self.0.inbox.drain(self.budget(), |item| {
            if panicked.is_some() {
                // Drop the rest of this turn's items, releasing their
                // scopes and barrier guards.
                return;
            }

            if let Err(payload) = self.process_item(item, handler) {
                panicked = Some(payload);
            }
        });

        if let Some(payload) = panicked {
            if more {
                self.discard();
            }

            resume_unwind(payload);
        }

        more
    }

    fn process_item(&self, item: Item<H>, handler: &mut H) -> thread::Result<()> {
        match item {
            Item::Deliver { delivery, handle } => {
                let ctx = EventContext {
                    scope: delivery.scope(),
                    mailbox: self,
                };

                catch_unwind(AssertUnwindSafe(|| {
                    handle(handler, delivery.payload(), &ctx)
                }))
                .map(|()| delivery.complete())
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
        H: Handler<E>,
    {
        const HANDLER_PREFIX: &str = "handler";

        let first = generate_metric_key::<H>(HANDLER_PREFIX);
        let second = format!("{}.{}", first, E::name());
        self.mailbox.subscribe::<E>().with_name(second)
    }
}
