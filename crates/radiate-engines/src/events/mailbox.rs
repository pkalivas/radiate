use crate::{
    Event, EventHandler, EventId, EventScope, Handler,
    events::{
        EventStream, Subscription, SubscriptionId,
        stream::{Delivery, Shared},
    },
};
use radiate_core::{Executor, Inbox, WaitGuard, error::RadiateResult};
use radiate_utils::{generate_metric_key, sentry_id};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{Arc, Mutex, Weak};

sentry_id!(MailBoxId);

/// Deliveries a [`MailBox`] drains per turn on a worker pool before handing the
/// worker back to the executor. On a serial executor the drain just keeps going.
const PROCESS_BATCH_SIZE: usize = 64;

/// The untyped face of a mailbox. Everything the stream can ask of a mailbox
/// without knowing its handler type.
pub(super) trait Recipient: Send + Sync {
    fn id(&self) -> MailBoxId;

    fn drain(&self, guard: WaitGuard);
}

/// Monomorphized per (E, H) and stored as a fn pointer, so one queue can hold
/// deliveries of every event type the handler subscribes to.
type HandleFn<H> = fn(&mut H, &Delivery, &EventContext<'_, H>);

fn handle<E, H>(handler: &mut H, delivery: &Delivery, ctx: &EventContext<'_, H>)
where
    E: Event,
    H: Handler<E>,
{
    if let Some(event) = delivery.event::<E>() {
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

    /// The type-erased handle the stream keeps for barriers.
    pub(super) fn recipient(&self) -> Arc<dyn Recipient> {
        Arc::new(self.clone())
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
            let scope = EventScope::default();
            let ctx = EventContext {
                scope: &scope,
                mailbox: self,
            };
            self.0.handler.lock().unwrap().start(&ctx)
        };

        if claimed {
            self.process();
        }

        result
    }

    pub(super) fn deliver<E>(&self, delivery: Delivery)
    where
        E: Event,
        H: Handler<E>,
    {
        self.enqueue(Item::Deliver {
            delivery,
            handle: handle::<E, H>,
        });
    }

    fn enqueue(&self, item: Item<H>) {
        if self.0.inbox.push(item) {
            self.schedule();
        }
    }

    /// Start a drain on the executor. On a serial executor this runs inline.
    fn schedule(&self) {
        let mailbox = self.clone();
        self.0.executor.submit(move || mailbox.process());
    }

    fn budget(&self) -> usize {
        if self.0.executor.is_parallel() {
            PROCESS_BATCH_SIZE
        } else {
            usize::MAX
        }
    }

    fn process(&self) {
        let more = {
            let Ok(mut handler) = self.0.handler.lock() else {
                self.discard();
                panic!(
                    "event handler `{}` panicked on an earlier event",
                    std::any::type_name::<H>()
                );
            };

            let mut panicked = None;
            let more = self.0.inbox.drain(self.budget(), |item| {
                if panicked.is_some() {
                    // Drop the rest of this turn's items, releasing their
                    // scopes and barrier guards.
                    return;
                }

                match item {
                    Item::Deliver { delivery, handle } => {
                        let ctx = EventContext {
                            scope: delivery.scope(),
                            mailbox: self,
                        };

                        let outcome = catch_unwind(AssertUnwindSafe(|| {
                            handle(&mut handler, &delivery, &ctx)
                        }));

                        match outcome {
                            Ok(()) => delivery.complete(),
                            Err(payload) => panicked = Some(payload),
                        }
                    }
                    Item::Drain(guard) => drop(guard),
                }
            });

            if let Some(payload) = panicked {
                if more {
                    self.discard();
                }

                resume_unwind(payload);
            }

            more
        };

        if more {
            self.schedule();
        }
    }

    /// Drop everything queued and release the claim. Must hold the claim.
    fn discard(&self) {
        self.0.inbox.drain(usize::MAX, drop);
    }
}

impl<H: Send + 'static> Recipient for MailBox<H> {
    fn id(&self) -> MailBoxId {
        self.0.id
    }

    fn drain(&self, guard: WaitGuard) {
        self.enqueue(Item::Drain(guard));
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
            stream.publish_scoped(event, self.scope.clone());
        }
    }

    /// The id of the event that started the chain this handler is running in.
    pub fn root_id(&self) -> EventId {
        self.scope.id()
    }

    pub fn subscribe<E>(&self) -> Subscription
    where
        E: Event,
        H: Handler<E>,
    {
        const HANDLER_PREFIX: &str = "handler";

        let first = generate_metric_key::<H>(HANDLER_PREFIX);
        let second = format!("{}.{}", first, E::name());
        self.mailbox.subscribe::<E>().named(second)
    }
}
