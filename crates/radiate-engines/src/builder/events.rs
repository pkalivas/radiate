use crate::{
    GeneticEngineBuilder,
    message::{EngineMessage, EventHandler},
};
use radiate_core::Chromosome;

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    pub fn subscribe<M>(self, handler: impl EventHandler<M> + Send + Sync + 'static) -> Self
    where
        M: EngineMessage + Send + Sync + 'static,
    {
        self.params.event_bus.subscribe(handler);
        self
    }
}
