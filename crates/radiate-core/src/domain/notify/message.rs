use super::actor::ActorId;
use crate::{MessageBroker, ThreadSync};
use radiate_utils::sentry_id;
use std::{any::Any, sync::Arc};
use std::{any::TypeId, sync::atomic::AtomicU64};

sentry_id!(EventId);

/// Fired by [`MessageBroker::subscribe`] every time a new actor is
/// registered — a lifecycle fact about the broker itself, independent of
/// whatever domain-specific messages (`EngineStart`, `Log`, ...) are
/// actually flowing through it. `subscriber_count` is the number of actors
/// now registered for `message_type`, including the one that just joined.
///
/// Subscribing to `ActorSubscribed` itself is not a special case: it goes
/// through the same `subscribe()` path as anything else, so the very call
/// that registers your `ActorSubscribed` listener immediately fires one
/// event describing that registration.
#[derive(Clone, Debug)]
pub struct ActorSubscribed {
    pub message_type: &'static str,
    pub actor_id: ActorId,
    pub subscriber_count: usize,
}

/// Fired when an actor's handler panics while processing a message. See
/// [`super::actor::Actor::drain`] for why the panic is caught in place
/// (before the `MutexGuard` around the handler would be dropped mid-unwind)
/// rather than being left to poison that actor's `Mutex` and silently kill
/// it for the rest of the process — the actor keeps handling later
/// messages after this fires. `panic_message` is best-effort: only
/// `&str`/`String` panic payloads become readable text, anything else
/// becomes a generic message.
///
/// Not re-emitted for a panic that happens while handling an
/// `ActorPanicked` itself — without that cutoff, a subscriber whose own
/// `ActorPanicked` handler always panics would flood the bus with an
/// unbounded chain of `ActorPanicked`-about-`ActorPanicked` events.
#[derive(Clone, Debug)]
pub struct ActorPanicked {
    pub message_type: &'static str,
    pub actor_id: ActorId,
    pub panic_message: String,
}

/// Anything that can ride the bus. Blanket-implemented — the only real
/// requirement is being safe to hand across the `Executor`'s worker threads.
pub trait Message: Send + Sync + 'static {}
impl<M: Send + Sync + 'static> Message for M {}

pub struct AnyEnvelope {
    payload: Box<dyn Any + Send>,
    type_id: TypeId,
}

impl AnyEnvelope {
    pub fn new<M: Message>(message: M) -> Self {
        AnyEnvelope {
            payload: Box::new(message),
            type_id: TypeId::of::<M>(),
        }
    }

    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    pub fn downcast_ref<M: Message>(&self) -> Option<&M> {
        self.payload.downcast_ref::<M>()
    }
}

/// A cheaply-clonable message envelope: wraps `D` in an `Arc` so fanning a
/// message out to many subscribed actors clones a pointer per actor, not the
/// payload itself. Most concrete message types on the bus should be a type
/// alias over this rather than hand-rolling their own `Arc` wrapper.
pub struct Envelope<D>(Arc<D>);

impl<D> Envelope<D> {
    pub fn new(data: D) -> Self {
        Envelope(Arc::new(data))
    }
}

impl<D> Clone for Envelope<D> {
    fn clone(&self) -> Self {
        Envelope(Arc::clone(&self.0))
    }
}

impl<D> std::ops::Deref for Envelope<D> {
    type Target = D;

    fn deref(&self) -> &D {
        &self.0
    }
}

/// Handed to every actor alongside the message it's processing. Carries the
/// same `ThreadSync` the owning engine (or whatever else set up this
/// `ActorSystem`) uses for pause/stop/step — so any actor can act back on
/// the thing it's observing, not just read it. Also carries the
/// `ActorSystem` itself, so a handler can publish further messages (e.g.
/// escalating a `Warn` into an `Error` after it's seen enough of them) —
/// see [`EventContext::send`].
///
/// `send`ing the same message type a handler is currently subscribed to is
/// a footgun: it won't blow the stack (dispatch goes through the mailbox,
/// not direct recursion) but it will happily flood the queue forever if
/// the handler doesn't have its own stopping condition.
#[derive(Clone)]
pub struct EventContext {
    pub(crate) sync: ThreadSync,
    pub(crate) system: MessageBroker,
    pub(crate) id: EventId,
}

impl EventContext {
    pub fn new(sync: ThreadSync, system: MessageBroker) -> Self {
        Self {
            sync,
            system,
            id: EventId::new(),
        }
    }

    pub fn id(&self) -> EventId {
        self.id
    }

    pub fn stop(&self) {
        self.sync.stop();
    }

    pub fn pause(&self) {
        self.sync.set_paused(true);
    }

    pub fn resume(&self) {
        self.sync.set_paused(false);
    }

    pub fn is_paused(&self) -> bool {
        self.sync.is_paused()
    }

    pub fn is_stopped(&self) -> bool {
        self.sync.is_stopped()
    }

    pub fn thread_id(&self) -> std::thread::ThreadId {
        std::thread::current().id()
    }

    /// Publish another message onto the same `ActorSystem` this handler is
    /// running on. Only actors subscribed to `M` are touched, same as
    /// [`ActorSystem::send`].
    pub fn send<M: Message>(&self, message: M) {
        self.system.trace_send(message, self.id);
    }

    /// Cheap check for whether anyone is subscribed to `M` before doing the
    /// work to build one. Same rationale as [`ActorSystem::has_subscribers`].
    pub fn has_subscribers<M: Message>(&self) -> bool {
        self.system.has_subscribers::<M>()
    }
}
