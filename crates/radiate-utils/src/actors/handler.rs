use crate::{Actor, Addr, MessageHandler};

pub struct FnActor<M: Send> {
    pub(crate) handler: Box<dyn FnMut(M, &Addr<Self>) + Send + Sync>,
}

impl<M: Send> Actor for FnActor<M> {}

impl<M: Send + 'static> MessageHandler<M> for FnActor<M> {
    fn handle(&mut self, message: M, ctx: &Addr<Self>) {
        (self.handler)(message, ctx);
    }
}
