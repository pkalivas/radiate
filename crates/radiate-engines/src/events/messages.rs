use crate::{
    Generation, Limit,
    context::EvolutionContext,
    events::{Message, SubscriptionId, addr::ActorId},
};
use radiate_core::{Chromosome, EngineState, MetricSet, Objective, Score, SmallStr};
use std::{fmt::Debug, sync::Arc};

impl Message for EngineState {
    type Response = ();
}

#[derive(Debug)]
pub enum StreamEvent {
    HandlerRegistered(SmallStr, ActorId),
    SubscriptionAdded(SmallStr, ActorId, SubscriptionId),
    FnHandler(SubscriptionId),
}

impl Message for StreamEvent {
    type Response = ();
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CheckpointSaved {
    pub index: usize,
    pub path: String,
}

impl Message for CheckpointSaved {
    type Response = ();
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Warning(pub String);

impl Message for Warning {
    type Response = ();
}

#[derive(Clone, Debug)]
pub struct LimitTriggered(pub usize, pub Limit);
impl Message for LimitTriggered {
    type Response = ();
}
#[derive(Clone, Debug)]
pub struct EpochStart(pub usize);

impl Message for EpochStart {
    type Response = ();
}

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
    pub best: T,
    pub score: Score,
}

impl<T> Message for Improvement<T>
where
    T: Send + Sync + 'static,
{
    type Response = ();
}

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
            best: ctx.best.clone(),
            score: ctx.score.clone().unwrap_or_default(),
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

impl<T> Message for EpochComplete<T>
where
    T: Send + Sync + 'static,
{
    type Response = ();
}

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

#[derive(Clone, Debug)]
pub struct EngineStart;

impl Message for EngineStart {
    type Response = ();
}

#[derive(Clone)]
pub struct EngineStop<T> {
    pub index: usize,
    pub best: T,
    pub metrics: MetricSet,
    pub score: Score,
}

impl<T> Message for EngineStop<T>
where
    T: Send + Sync + 'static,
{
    type Response = ();
}

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

impl<C, T> Message for GenerationSnapshot<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + Sync + 'static,
{
    type Response = ();
}

impl<C, T> From<&EvolutionContext<C, T>> for GenerationSnapshot<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + Sync + 'static,
    Generation<C, T>: for<'a> From<&'a EvolutionContext<C, T>>,
{
    fn from(ctx: &EvolutionContext<C, T>) -> Self {
        GenerationSnapshot {
            generation: Arc::new(Generation::from(&ctx)),
        }
    }
}
