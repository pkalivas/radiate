use crate::{
    EventHandler, EventType, GeneticEngineBuilder, OnEpochComplete, OnEpochStart, OnImprovement,
    OnStart, OnStop,
};
use radiate_core::Chromosome;

impl<C, T> GeneticEngineBuilder<C, T>
where
    C: Chromosome + PartialEq + Clone,
    T: Clone + Send,
{
    /// Subscribe to engine events with the given event handler.
    /// The event handler will be called whenever an event is emitted by the engine.
    /// You can use this to log events, or to perform custom actions
    /// based on the events emitted by the engine.
    pub fn subscribe<H>(mut self, handler: H) -> Self
    where
        H: EventHandler<T> + 'static,
    {
        self.params.bus.subscribe(handler);
        self
    }

    pub fn subscribe_to<H>(mut self, event_type: EventType, handler: H) -> Self
    where
        H: EventHandler<T> + 'static,
    {
        self.params.bus.subscribe_typed(event_type, handler);
        self
    }

    pub fn on_start<H>(mut self, handler: H) -> Self
    where
        H: OnStart + 'static,
        T: 'static,
    {
        self.params.bus.on_start(handler);
        self
    }

    pub fn on_stop<H>(mut self, handler: H) -> Self
    where
        H: OnStop<T> + 'static,
        T: 'static,
    {
        self.params.bus.on_stop(handler);
        self
    }

    pub fn on_epoch_start<H>(mut self, handler: H) -> Self
    where
        H: OnEpochStart<T> + 'static,
        T: 'static,
    {
        self.params.bus.on_epoch_start(handler);
        self
    }

    pub fn on_epoch_complete<H>(mut self, handler: H) -> Self
    where
        H: OnEpochComplete<T> + 'static,
        T: 'static,
    {
        self.params.bus.on_epoch_complete(handler);
        self
    }

    pub fn on_improvement<H>(mut self, handler: H) -> Self
    where
        H: OnImprovement<T> + 'static,
        T: 'static,
    {
        self.params.bus.on_improvement(handler);
        self
    }
}
