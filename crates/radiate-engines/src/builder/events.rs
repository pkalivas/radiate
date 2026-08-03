use crate::{
    EngineEvent, EpochCompleted, EpochStarted, EventHandler, GeneticEngineBuilder, Improved,
    Started, Stopped,
};
use radiate_core::Chromosome;

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
        self.params.bus.subscribe(handler);
        self
    }

    pub fn on_start<H>(self, handler: H) -> Self
    where
        H: EventHandler<Started> + 'static,
    {
        self.params.bus.on_start(handler);
        self
    }

    pub fn on_stop<H>(self, handler: H) -> Self
    where
        H: EventHandler<Stopped<T>> + 'static,
        T: Send + Sync + 'static,
    {
        self.params.bus.on_stop(handler);
        self
    }

    pub fn on_epoch_start<H>(self, handler: H) -> Self
    where
        H: EventHandler<EpochStarted> + 'static,
    {
        self.params.bus.on_epoch_start(handler);
        self
    }

    pub fn on_epoch_complete<H>(self, handler: H) -> Self
    where
        H: EventHandler<EpochCompleted<T>> + 'static,
        T: Send + Sync + 'static,
    {
        self.params.bus.on_epoch_complete(handler);
        self
    }

    pub fn on_improvement<H>(self, handler: H) -> Self
    where
        H: EventHandler<Improved<T>> + 'static,
        T: Send + Sync + 'static,
    {
        self.params.bus.on_improvement(handler);
        self
    }
}
