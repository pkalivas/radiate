use crate::{
    GeneticEngineBuilder,
    message::{Event, EventHandler},
};
use radiate_core::Chromosome;

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    pub fn subscribe<M>(self, handler: impl EventHandler<M>) -> Self
    where
        M: Event,
    {
        self.params.event_bus.subscribe(handler);
        self
    }
}
