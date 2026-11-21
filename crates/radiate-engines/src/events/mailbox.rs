use crate::{
    Event, EventId, WeakScope,
    events::{
        event::Payload,
        stream::{Shared, StreamMessage},
    },
};
use crate::{
    EventHandler,
    events::{EventStream, Subscription, SubscriptionId},
};
use crate::{Handler, events::subscription::Counters};
use radiate_core::{Executor, Inbox, error::RadiateResult};
use radiate_utils::{generate_metric_key, sentry_id};
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::sync::{Arc, Mutex, Weak};

sentry_id!(MailBoxId);

/// Deliveries a [`MailBox`] drains per turn on a worker pool before handing the
/// worker back to the executor. On a serial executor the drain just keeps going.
const PROCESS_BATCH_SIZE: usize = 64;

/// Downcast and hand a payload to `H`'s handler for `E`. Stored as a plain fn
/// pointer in each [`Envelope`], so queueing costs no extra allocation.
fn deliver<E, H>(handler: &mut H, payload: &Payload, ctx: &EventContext<'_, H>)
where
    E: Event,
    H: Handler<E>,
{
    // The stream routes by TypeId, so this always matches.
    if let Some(event) = payload.downcast_ref::<E>() {
        handler.handle(event, ctx);
    }
}

/// One queued delivery: the published event plus everything needed to hand it
/// to the handler once this mailbox's drain reaches it.
struct Envelope<H> {
    payload: StreamMessage,
    counters: Arc<Counters>,
    deliver: fn(&mut H, &Payload, &EventContext<'_, H>),
}

struct Inner<H> {
    id: MailBoxId,
    handler: Mutex<H>,
    inbox: Inbox<Envelope<H>>,
    executor: Arc<Executor>,
    stream: Weak<Shared>,
}

/// A handler plus its queue of pending deliveries.
///
/// Every event routed to the handler is pushed onto its [`Inbox`], and at most
/// one drain runs at a time, inline on a serial executor or as a single job on a
/// worker pool. So a handler sees its events in publish order, never runs on two
/// threads at once, and never parks pool workers waiting on its lock.
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

    /// Run the handler's `start`. The inbox is claimed for the duration, so any
    /// event `start` publishes to this handler (or that arrives from elsewhere
    /// once `start` has subscribed) queues up and is drained right after.
    pub(super) fn start(&self) -> RadiateResult<()>
    where
        H: EventHandler,
    {
        let claimed = self.0.inbox.try_claim();
        let result = {
            let trace = WeakScope::default();
            let ctx = EventContext {
                id: EventId::new(),
                mailbox: self,
                context: &trace,
            };
            self.0.handler.lock().unwrap().start(&ctx)
        };

        if claimed {
            self.process();
        }

        result
    }

    pub(super) fn deliver<E>(&self, payload: StreamMessage, counters: Arc<Counters>)
    where
        E: Event,
        H: Handler<E>,
    {
        let envelope = Envelope {
            payload,
            counters,
            deliver: deliver::<E, H>,
        };

        if self.0.inbox.push(envelope) {
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
        let mut shutdown = None;
        let more = {
            let Ok(mut handler) = self.0.handler.lock() else {
                self.discard();
                panic!(
                    "event handler `{}` panicked on an earlier event",
                    std::any::type_name::<H>()
                );
            };

            let mut panicked = None;
            let more = self.0.inbox.drain(self.budget(), |envelope| {
                let outcome = match &envelope.payload {
                    StreamMessage::Event {
                        id,
                        payload,
                        scope: trace,
                    } => {
                        let ctx = EventContext {
                            id: *id,
                            mailbox: self,
                            context: trace,
                        };
                        catch_unwind(AssertUnwindSafe(|| {
                            (envelope.deliver)(&mut handler, payload, &ctx)
                        }))
                    }
                    StreamMessage::Wait(guard) => {
                        shutdown = Some(guard.clone());
                        Ok(())
                    }
                };

                match outcome {
                    Ok(()) => envelope.counters.record_handled(),
                    Err(payload) => {
                        panicked = Some(payload);
                        return;
                    }
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

        drop(shutdown);

        if more {
            // Only reachable on a pool: give the worker back, continue as a new job
            self.schedule();
        }
    }

    /// Drop everything queued and release the claim. Must hold the claim.
    fn discard(&self) {
        self.0.inbox.drain(usize::MAX, drop);
    }
}

impl<H> Clone for MailBox<H> {
    fn clone(&self) -> Self {
        MailBox(Arc::clone(&self.0))
    }
}

pub struct EventContext<'a, H> {
    id: EventId,
    mailbox: &'a MailBox<H>,
    context: &'a WeakScope,
}

impl<H> EventContext<'_, H> {
    /// Publish an event as part of the chain currently being handled. If that
    /// chain is being waited on (for example by [`EventStream::flush`]), the wait
    /// also covers this event and everything it causes.
    ///
    /// Does nothing if the stream has already been dropped.
    pub fn publish<E: Event>(&self, event: E) {
        if let Some(stream) = self.mailbox.stream() {
            stream.publish_scoped(event, self.context.clone());
        }
    }

    /// The id of the event that started the chain this handler is running in.
    pub fn root_id(&self) -> EventId {
        self.id
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
