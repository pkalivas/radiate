use crate::{Actor, Addr, MessageHandler};

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

pub trait ActorHandleFn<M, A: Actor>: Send + Sync {
    fn handle(&mut self, message: M, ctx: &Addr<A>);
}

impl<M, A, F> ActorHandleFn<M, A> for F
where
    F: FnMut(M, &Addr<A>) + Send + Sync,
    M: Send + 'static,
    A: Actor + 'static,
{
    fn handle(&mut self, message: M, ctx: &Addr<A>) {
        self(message, ctx)
    }
}

pub struct FnActor<M: Send> {
    pub(crate) handler: Box<dyn FnMut(M, &Addr<Self>) + Send + Sync>,
}

impl<M: Send> Actor for FnActor<M> {}

impl<M: Send + 'static> MessageHandler<M> for FnActor<M> {
    fn handle(&mut self, message: M, ctx: &Addr<Self>) {
        (self.handler)(message, ctx);
    }
}
