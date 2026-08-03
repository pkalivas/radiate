use crate::ThreadSync;

/// Handed to every actor alongside the message it's processing. Carries the
/// same `ThreadSync` the owning engine (or whatever else set up this
/// `ActorSystem`) uses for pause/stop/step — so any actor can act back on
/// the thing it's observing, not just read it.
#[derive(Clone)]
pub struct EventContext {
    pub sync: ThreadSync,
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
