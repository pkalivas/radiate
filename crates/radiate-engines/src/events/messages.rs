use crate::{
    Event, Generation, Limit, StreamStats, Subscription, SubscriptionInfo,
    context::EvolutionContext,
};
use radiate_core::{
    Chromosome, EngineState, Front, MetricSet, Objective, Phenotype, Score, SmallStr,
};
use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

#[derive(Clone, Debug)]
pub struct DeferredEvent<E>(pub E);

impl<E: Event> Event for DeferredEvent<E> {
    fn name() -> SmallStr {
        SmallStr::from_string(format!("{}.deferred", E::name()))
    }
}

#[derive(Clone, Debug)]
pub enum StreamEvent {
    Added(Subscription),
    Removed(SubscriptionInfo),
    Stats(StreamStats),
    Drained(StreamStats),
}

impl Event for StreamEvent {}

#[derive(Clone, Debug)]
pub struct BuilderEvent(pub String);

impl Event for BuilderEvent {}

/// ---------------

#[derive(Clone, Debug)]
pub struct EngineStateChange {
    pub index: usize,
    pub from: EngineState,
    pub to: EngineState,
}

impl Event for EngineStateChange {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CheckpointSaved {
    pub index: usize,
    pub path: String,
}

impl Event for CheckpointSaved {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Warning(pub String);

impl Event for Warning {}

#[derive(Clone, Debug)]
pub struct LimitTriggered(pub usize, pub Limit);

impl Event for LimitTriggered {}

#[derive(Clone, Debug)]
pub struct EpochStart(pub usize);

impl Event for EpochStart {}

impl<C, T> From<&EvolutionContext<C, T>> for EpochStart
where
    C: Chromosome,
    T: Clone,
{
    fn from(ctx: &EvolutionContext<C, T>) -> Self {
        EpochStart(ctx.index)
    }
}

#[derive(Clone)]
pub struct Improvement<T> {
    pub index: usize,
    pub objective: Objective,
    pub best: T,
    pub score: Score,
    pub previous_score: Option<Score>,
}

impl<T> Event for Improvement<T> where T: Send + Sync + 'static {}

impl<T> Debug for Improvement<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Improved(index={}, score={:?})", self.index, self.score)
    }
}

impl<C, T> From<&EvolutionContext<C, T>> for Improvement<T>
where
    C: Chromosome,
    T: Clone,
{
    fn from(ctx: &EvolutionContext<C, T>) -> Self {
        Improvement {
            index: ctx.index,
            objective: ctx.objective.clone(),
            best: ctx.best.clone(),
            score: ctx.score.clone().unwrap_or_default(),
            previous_score: ctx.previous_score.clone(),
        }
    }
}

#[derive(Clone)]
pub struct EpochComplete<T> {
    pub index: usize,
    pub best: T,
    pub metrics: MetricSet,
    pub score: Score,
    pub objective: Objective,
}

impl<T> Event for EpochComplete<T> where T: Send + Sync + 'static {}

impl<C, T> From<&EvolutionContext<C, T>> for EpochComplete<T>
where
    C: Chromosome,
    T: Clone,
{
    fn from(ctx: &EvolutionContext<C, T>) -> Self {
        EpochComplete {
            index: ctx.index,
            best: ctx.best.clone(),
            metrics: ctx.metrics.clone(),
            score: ctx.score.clone().unwrap_or_default(),
            objective: ctx.objective.clone(),
        }
    }
}

impl<T> Debug for EpochComplete<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EpochCompleted(index={}, score={:?}, objective={:?})",
            self.index, self.score, self.objective
        )
    }
}

#[derive(Clone)]
pub struct EngineStart<C: Chromosome> {
    pub front: Arc<RwLock<Front<Phenotype<C>>>>,
}

impl<C: Chromosome + 'static> Event for EngineStart<C> {}

impl<C: Chromosome, T> From<&EvolutionContext<C, T>> for EngineStart<C>
where
    T: Clone,
{
    fn from(ctx: &EvolutionContext<C, T>) -> Self {
        EngineStart {
            front: Arc::clone(&ctx.front()),
        }
    }
}

impl<C: Chromosome> Debug for EngineStart<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EngineStart")
    }
}

#[derive(Clone)]
pub struct EngineStop<T> {
    pub index: usize,
    pub best: T,
    pub metrics: MetricSet,
    pub score: Score,
}

impl<T> Event for EngineStop<T> where T: Send + Sync + 'static {}

impl<C, T> From<&EvolutionContext<C, T>> for EngineStop<T>
where
    C: Chromosome,
    T: Clone,
{
    fn from(ctx: &EvolutionContext<C, T>) -> Self {
        EngineStop {
            index: ctx.index,
            best: ctx.best.clone(),
            metrics: ctx.metrics.clone(),
            score: ctx.score.clone().unwrap_or_default(),
        }
    }
}

impl<T> Debug for EngineStop<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EngineStopped(index={}, score={:?})",
            self.index, self.score
        )
    }
}

#[derive(Clone, Debug)]
pub struct GenerationSnapshot<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + Sync + 'static,
{
    pub generation: Arc<Generation<C, T>>,
}

impl<C, T> Event for GenerationSnapshot<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + Sync + 'static,
{
}

impl<C, T> From<&EvolutionContext<C, T>> for GenerationSnapshot<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + Sync + 'static,
    Generation<C, T>: for<'a> From<&'a EvolutionContext<C, T>>,
{
    fn from(ctx: &EvolutionContext<C, T>) -> Self {
        GenerationSnapshot {
            generation: Arc::new(Generation::from(ctx)),
        }
    }
}
