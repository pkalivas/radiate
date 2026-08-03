use crate::{
    EngineEvent, EpochCompleted, EventHandler, GeneticEngineBuilder,
    events::{EngineImproved, EngineMessage, EngineStart, EngineStopped, EpochStart},
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
        self.params.broker.subscribe(handler);
        self
    }

    pub fn on_start<H>(self, handler: H) -> Self
    where
        H: EventHandler<EngineStart> + 'static,
    {
        self.params.broker.subscribe(handler);
        self
    }

    pub fn on_stop<H>(self, handler: H) -> Self
    where
        H: EventHandler<EngineStopped<T>> + 'static,
        T: Send + Sync + 'static,
    {
        self.params.broker.subscribe(handler);
        self
    }

    pub fn on_epoch_start<H>(self, handler: H) -> Self
    where
        H: EventHandler<EpochStart> + 'static,
    {
        self.params.broker.subscribe(handler);
        self
    }

    pub fn on_epoch_complete<H>(self, handler: H) -> Self
    where
        H: EventHandler<EpochCompleted<T>> + 'static,
        T: Send + Sync + 'static,
    {
        self.params.broker.subscribe(handler);
        self
    }

    pub fn on_improvement<H>(self, handler: H) -> Self
    where
        H: EventHandler<EngineImproved<T>> + 'static,
        T: Send + Sync + 'static,
    {
        self.params.broker.subscribe(handler);
        self
    }

    /// Subscribe to a message type this engine doesn't know about (a custom
    /// `Warning`, a `LogMessage`, ...). It rides the same `ActorSystem` as
    /// the built-in engine events — same executor, same shared `ThreadSync`
    /// — but nothing here publishes `M`; that's on whoever owns it, via
    /// `ActorSystem::send` (or `EventContext::send` from inside a handler).
    pub fn subscribe_typed<M, H>(self, handler: H) -> Self
    where
        M: EngineMessage,
        H: EventHandler<M> + 'static,
    {
        self.params.broker.subscribe(handler);
        self
    }
}
