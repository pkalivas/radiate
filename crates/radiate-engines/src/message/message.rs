use crate::{Limit, context::EvolutionContext, message::LogEvent};
use radiate_core::{Chromosome, Ecosystem, MetricSet, Objective, Score};
use std::{fmt::Debug, sync::Arc};

mod sealed {
    pub trait Sealed {}
}

pub trait EngineMessage: sealed::Sealed + std::fmt::Debug + Clone {}

macro_rules! engine_message {
    ($($t:ty),* $(,)?) => { $(
        impl sealed::Sealed for $t {}
        impl EngineMessage for $t {}
    )* };
}
engine_message!(EpochStart, LimitTriggered, LogEvent, CheckpointSaved,);
impl<T: Clone + Send + Sync + 'static> sealed::Sealed for Improvement<T> {}
impl<T: Clone + Send + Sync + 'static> EngineMessage for Improvement<T> {}

impl<T: Clone + Send + Sync + 'static> sealed::Sealed for EpochComplete<T> {}
impl<T: Clone + Send + Sync + 'static> EngineMessage for EpochComplete<T> {}

impl<T: Clone + Send + Sync + 'static> sealed::Sealed for EngineStop<T> {}
impl<T: Clone + Send + Sync + 'static> EngineMessage for EngineStop<T> {}

impl<C: Chromosome + Clone> sealed::Sealed for EcosystemSnapshot<C> {}
impl<C: Chromosome + Clone + Send + Sync + 'static> EngineMessage for EcosystemSnapshot<C> {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CheckpointSaved {
    pub index: usize,
    pub path: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Warning(pub String);

#[derive(Clone, Debug)]
pub struct LimitTriggered(pub usize, pub Limit);

#[derive(Clone, Debug)]
pub struct EpochStart(pub usize);

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
pub struct EngineStop<T> {
    pub index: usize,
    pub best: T,
    pub metrics: MetricSet,
    pub score: Score,
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

#[derive(Clone)]
pub struct EcosystemSnapshot<C: Chromosome + 'static> {
    pub index: usize,
    pub ecosystem: Arc<Ecosystem<C>>,
}

impl<C: Chromosome + Clone, T> From<&EvolutionContext<C, T>> for EcosystemSnapshot<C> {
    fn from(ctx: &EvolutionContext<C, T>) -> Self {
        EcosystemSnapshot {
            index: ctx.index,
            ecosystem: Arc::new(ctx.ecosystem.clone()),
        }
    }
}

impl<C: Chromosome> Debug for EcosystemSnapshot<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EcosystemSnapshot(index={})", self.index)
    }
}
