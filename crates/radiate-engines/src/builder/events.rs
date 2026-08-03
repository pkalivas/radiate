use crate::{EventHandler, GeneticEngineBuilder, events::EngineMessage};
use radiate_core::Chromosome;

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    pub fn subscribe<H, E>(self, handler: H) -> Self
    where
        H: EventHandler<E> + 'static,
        E: EngineMessage,
    {
        self.params.broker.subscribe(handler);
        self
    }
}
