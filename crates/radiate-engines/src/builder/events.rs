use crate::{Addr, EventHandler, GeneticEngineBuilder, actors::FnActor, events::EngineMessage};
use radiate_core::Chromosome;

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    pub fn subscribe<M>(self, mut handler: impl EventHandler<M> + Send + Sync + 'static) -> Self
    where
        M: EngineMessage + Send + Sync + 'static,
    {
        let handler = move |message: M, _ctx: &Addr<FnActor<M>>| {
            handler.handle(&message);
        };

        self.params.actor_system.subscribe(handler);
        self
    }
    // pub fn subscribe<M>(self, handler: impl Fn(&M) + Send + Sync + 'static) -> Self
    // where
    //     M: EngineMessage + Send + Sync + 'static,
    // {
    //     let handler = move |message: M, _ctx: &Addr<FnActor<M>>| {
    //         handler(&message);
    //     };

    //     self.params.actor_system.subscribe(handler);
    //     self
    // }
}
