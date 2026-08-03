use crate::context::EvolutionContext;
use radiate_core::{Chromosome, Envelope, MetricSet, Objective, Score};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct LimitTriggered {
    pub generation: usize,
    pub kind: &'static str,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct EngineStart;

#[derive(Clone, Debug)]
pub struct EpochStart {
    pub index: usize,
}

impl<C, T> From<&EvolutionContext<C, T>> for EpochStart
where
    C: Chromosome,
    T: Clone,
{
    fn from(ctx: &EvolutionContext<C, T>) -> Self {
        EpochStart { index: ctx.index }
    }
}

#[derive(Clone, Debug)]
pub struct EngineImproved<T> {
    pub index: usize,
    pub best: T,
    pub score: Score,
}

impl<C, T> From<&EvolutionContext<C, T>> for EngineImproved<T>
where
    C: Chromosome,
    T: Clone,
{
    fn from(ctx: &EvolutionContext<C, T>) -> Self {
        EngineImproved {
            index: ctx.index,
            best: ctx.best.clone(),
            score: ctx.score.clone().unwrap_or_default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EpochCompleted<T> {
    pub index: usize,
    pub best: T,
    pub metrics: MetricSet,
    pub score: Score,
    pub objective: Objective,
}

impl<C, T> From<&EvolutionContext<C, T>> for EpochCompleted<T>
where
    C: Chromosome,
    T: Clone,
{
    fn from(ctx: &EvolutionContext<C, T>) -> Self {
        EpochCompleted {
            index: ctx.index,
            best: ctx.best.clone(),
            metrics: ctx.metrics.clone(),
            score: ctx.score.clone().unwrap_or_default(),
            objective: ctx.objective.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct EngineStopped<T> {
    pub index: usize,
    pub best: T,
    pub metrics: MetricSet,
    pub score: Score,
}

impl<C, T> From<&EvolutionContext<C, T>> for EngineStopped<T>
where
    C: Chromosome,
    T: Clone,
{
    fn from(ctx: &EvolutionContext<C, T>) -> Self {
        EngineStopped {
            index: ctx.index,
            best: ctx.best.clone(),
            metrics: ctx.metrics.clone(),
            score: ctx.score.clone().unwrap_or_default(),
        }
    }
}

#[derive(Clone)]
pub enum EngineEvent<T> {
    Started(EngineStart),
    Stopped(EngineStopped<T>),
    EpochStarted(EpochStart),
    EpochCompleted(EpochCompleted<T>),
    Improved(EngineImproved<T>),
    LimitTriggered(LimitTriggered),
    LogInfo(String),
    LogWarn(String),
}

impl<T> EngineEvent<T> {
    pub fn is_start(&self) -> bool {
        matches!(self, EngineEvent::Started(_))
    }

    pub fn is_stop(&self) -> bool {
        matches!(self, EngineEvent::Stopped(_))
    }

    pub fn is_epoch_start(&self) -> bool {
        matches!(self, EngineEvent::EpochStarted(_))
    }

    pub fn is_epoch_complete(&self) -> bool {
        matches!(self, EngineEvent::EpochCompleted(_))
    }

    pub fn is_improvement(&self) -> bool {
        matches!(self, EngineEvent::Improved(_))
    }
}

impl<T: Debug> Debug for EngineEvent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineEvent::Started(_) => write!(f, "Started"),
            EngineEvent::Stopped(s) => write!(
                f,
                "Stopped(index={}, best={:?}, score={:?})",
                s.index, s.best, s.score
            ),
            EngineEvent::EpochStarted(s) => write!(f, "EpochStarted(index={})", s.index),
            EngineEvent::EpochCompleted(s) => write!(
                f,
                "EpochCompleted(index={}, best={:?}, score={:?}, objective={:?})",
                s.index, s.best, s.score, s.objective
            ),
            EngineEvent::Improved(s) => write!(
                f,
                "Improved(index={}, best={:?}, score={:?})",
                s.index, s.best, s.score
            ),
            EngineEvent::LimitTriggered(l) => write!(
                f,
                "LimitTriggered(generation={}, kind={}, description={})",
                l.generation, l.kind, l.description
            ),
            EngineEvent::LogInfo(msg) => write!(f, "LogInfo({})", msg),
            EngineEvent::LogWarn(msg) => write!(f, "LogWarn({})", msg),
        }
    }
}
