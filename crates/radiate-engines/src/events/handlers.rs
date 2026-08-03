use crate::events::message::*;
use radiate_core::{MetricSet, Objective, Score};

pub trait EventHandler<T>: Send + Sync {
    fn handle(&mut self, event: EngineEvent<T>);
}

pub trait OnStart: Send + Sync {
    fn on_start(&mut self);
}

pub trait OnStop<T>: Send + Sync {
    fn on_stop(&mut self, event: OnStopEvent<'_, T>);
}

pub trait OnEpochStart<T>: Send + Sync {
    fn on_epoch_start(&mut self, event: OnEpochStartEvent);
}

pub trait OnEpochComplete<T>: Send + Sync {
    fn on_epoch_complete(&mut self, event: OnEpochEvent<'_, T>);
}

pub trait OnImprovement<T>: Send + Sync {
    fn on_improvement(&mut self, event: OnImprovementEvent<'_, T>);
}

impl<T, F> EventHandler<T> for F
where
    F: Fn(&EngineEvent<T>) + Send + Sync,
{
    fn handle(&mut self, event: EngineEvent<T>) {
        self(&event)
    }
}

impl<F> OnStart for F
where
    F: Fn() + Send + Sync,
{
    fn on_start(&mut self) {
        self()
    }
}

impl<T, F> OnStop<T> for F
where
    F: Fn(OnStopEvent<'_, T>) + Send + Sync,
{
    fn on_stop(&mut self, event: OnStopEvent<'_, T>) {
        self(event)
    }
}

impl<T, F> OnEpochStart<T> for F
where
    F: Fn(OnEpochStartEvent) + Send + Sync,
{
    fn on_epoch_start(&mut self, event: OnEpochStartEvent) {
        self(event)
    }
}

impl<T, F> OnEpochComplete<T> for F
where
    F: Fn(OnEpochEvent<'_, T>) + Send + Sync,
{
    fn on_epoch_complete(&mut self, event: OnEpochEvent<'_, T>) {
        self(event)
    }
}

/// Adapters bridging the single-purpose `OnXxx` traits onto the generic
/// `EventHandler<T>` the bus actually stores. Kept as newtypes rather than
/// blanket `impl<T, H: OnStart> EventHandler<T> for H` because that would
/// overlap with the `Fn(&EngineEvent<T>)` blanket impl above.
pub(crate) struct OnStartAdapter<H>(pub H);

impl<T, H: OnStart> EventHandler<T> for OnStartAdapter<H> {
    fn handle(&mut self, event: EngineEvent<T>) {
        if let EngineEventInner::Start = event.inner() {
            self.0.on_start();
        }
    }
}

pub(crate) struct OnStopAdapter<H>(pub H);

pub struct OnStopEvent<'a, T> {
    pub index: usize,
    pub best: &'a T,
    pub metrics: &'a MetricSet,
    pub score: &'a Score,
}

impl<T, H: OnStop<T>> EventHandler<T> for OnStopAdapter<H> {
    fn handle(&mut self, event: EngineEvent<T>) {
        if let EngineEventInner::Stop(index, best, metrics, score) = event.inner() {
            let stop_event = OnStopEvent {
                index: *index,
                best,
                metrics,
                score,
            };
            self.0.on_stop(stop_event);
        }
    }
}

pub(crate) struct OnEpochStartAdapter<H>(pub H);

pub struct OnEpochStartEvent {
    pub index: usize,
}

impl<T, H: OnEpochStart<T>> EventHandler<T> for OnEpochStartAdapter<H> {
    fn handle(&mut self, event: EngineEvent<T>) {
        if let EngineEventInner::EpochStart(index) = event.inner() {
            self.0.on_epoch_start(OnEpochStartEvent { index: *index });
        }
    }
}

pub(crate) struct OnEpochCompleteAdapter<H>(pub H);

pub struct OnEpochEvent<'a, T> {
    pub index: usize,
    pub best: &'a T,
    pub metrics: &'a MetricSet,
    pub score: &'a Score,
    pub objective: Objective,
}

impl<T, H: OnEpochComplete<T>> EventHandler<T> for OnEpochCompleteAdapter<H> {
    fn handle(&mut self, event: EngineEvent<T>) {
        if let EngineEventInner::EpochComplete(index, best, metrics, score, objective) =
            event.inner()
        {
            let epoch_event = OnEpochEvent {
                index: *index,
                best,
                metrics,
                score,
                objective: objective.clone(),
            };
            self.0.on_epoch_complete(epoch_event);
        }
    }
}

pub(crate) struct OnImprovementAdapter<H>(pub H);

pub struct OnImprovementEvent<'a, T> {
    pub index: usize,
    pub best: &'a T,
    pub score: &'a Score,
}

impl<T, H: OnImprovement<T>> EventHandler<T> for OnImprovementAdapter<H> {
    fn handle(&mut self, event: EngineEvent<T>) {
        if let EngineEventInner::Improvement(index, best, score) = event.inner() {
            let improvement_event = OnImprovementEvent {
                index: *index,
                best,
                score,
            };
            self.0.on_improvement(improvement_event);
        }
    }
}
