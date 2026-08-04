use crate::{Actor, actors::system::ActorContext};

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
    type Message = M;

    fn receive(&mut self, message: Self::Message, ctx: &ActorContext) {
        (self.handler)(message, ctx);
    }

    fn on_child_failure(&mut self, _: String) {}
}
