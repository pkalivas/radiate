use crate::{Actor, MessageHandler, actors::context::ActorContext};

pub trait EventHandler<M>: Send + Sync {
    fn handle(&mut self, message: &M, ctx: &ActorContext);
}

impl<M, F> EventHandler<M> for F
where
    F: FnMut(&M, &ActorContext) + Send + Sync,
{
    fn handle(&mut self, message: &M, ctx: &ActorContext) {
        self(message, ctx)
    }
}

pub struct FnActor<M: Send> {
    pub(crate) handler: Box<dyn FnMut(M, &ActorContext) + Send + Sync>,
}

impl<M: Send> Actor for FnActor<M> {
    fn on_child_failure(&mut self, _: String) {}
}

impl<M: Send + 'static> MessageHandler<M> for FnActor<M> {
    fn handle(&mut self, message: M, ctx: &ActorContext) {
        (self.handler)(message, ctx);
    }
}
