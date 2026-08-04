use crate::{GeneticEngineBuilder, events::EngineMessage};
use radiate_core::{ActorContext, Chromosome};

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    pub fn subscribe<M>(self, handler: impl Fn(&M, &ActorContext) + Send + Sync + 'static) -> Self
    where
        M: EngineMessage + Send + Sync + 'static,
    {
        self.params.actor_system.subscribe(handler);
        self
    }
}
