use crate::{Actor, MessageHandler, actors::context::ActorContext};

pub trait EventHandler<M>: Send + Sync {
    fn handle(&mut self, message: &M);
}

impl<M, F> EventHandler<M> for F
where
    F: FnMut(&M) + Send + Sync,
{
    fn handle(&mut self, message: &M) {
        self(message)
    }
}

pub struct FnActor<M: Send> {
    pub(crate) handler: Box<dyn FnMut(M, &ActorContext) + Send + Sync>,
}

impl<M: Send> Actor for FnActor<M> {}

impl<M: Send + 'static> MessageHandler<M> for FnActor<M> {
    fn handle(&mut self, message: M, ctx: &ActorContext) {
        (self.handler)(message, ctx);
    }
}
