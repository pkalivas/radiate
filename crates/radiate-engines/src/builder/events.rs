use std::fmt::Debug;

use crate::{EngineEvent, EventHandler, GeneticEngineBuilder};
use radiate_core::{Chromosome, Message};

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    /// Subscribe to every kind of engine event with the given event handler.
    /// You can use this to log events, or to perform custom actions
    /// based on the events emitted by the engine.
    pub fn subscribe<H>(self, handler: H) -> Self
    where
        H: EventHandler<EngineEvent<T>> + 'static,
        T: Send + Sync + 'static,
    {
        self.params.broker.subscribe(handler);
        self
    }

    pub fn subscribe_to<H, E>(self, handler: H) -> Self
    where
        H: EventHandler<E> + 'static,
        E: Message + Debug,
    {
        self.params.broker.subscribe(handler);
        self
    }
}
