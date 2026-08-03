use crate::notify::message::EventContext;

pub trait EventHandler<M>: Send + Sync {
    fn handle(&mut self, message: &M, ctx: &EventContext);
}

impl<M, F> EventHandler<M> for F
where
    F: FnMut(&M, &EventContext) + Send + Sync,
{
    fn handle(&mut self, message: &M, ctx: &EventContext) {
        self(message, ctx)
    }
}
