use super::message::Message;
use super::system::ActorSystem;
use crate::ThreadSync;

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
    pub(crate) system: ActorSystem,
}

impl EventContext {
    pub fn new(sync: ThreadSync, system: ActorSystem) -> Self {
        Self { sync, system }
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
    pub fn send<M: Message + Clone>(&self, message: M) {
        self.system.send(message);
    }

    /// Cheap check for whether anyone is subscribed to `M` before doing the
    /// work to build one. Same rationale as [`ActorSystem::has_subscribers`].
    pub fn has_subscribers<M: Message>(&self) -> bool {
        self.system.has_subscribers::<M>()
    }
}

pub trait EventHandler<M>: Send + Sync {
    fn handle(&mut self, message: M, ctx: &EventContext);
}

impl<M, F> EventHandler<M> for F
where
    F: FnMut(M, &EventContext) + Send + Sync,
{
    fn handle(&mut self, message: M, ctx: &EventContext) {
        self(message, ctx)
    }
}
