use crate::context::EvolutionContext;
use radiate_core::{Chromosome, Message, MetricSet, Objective, Score};
use std::fmt::Debug;

mod sealed {
    pub trait Sealed {}
}

/// Marks a type as one of radiate-engines's own concrete message kinds —
/// the family `EngineEvent<T>` aggregates via the relay. Deliberately
/// *not* implemented for `EngineEvent<T>` itself: subscribing to the
/// wildcard only works once the relay is wired up, and that only happens
/// through `GeneticEngineBuilder::subscribe()` (pre-build). Sealing this
/// bound onto `GeneticEngine::subscribe` turns "subscribe to the wildcard
/// after `.build()`" from a silent no-op into a compile error — you
/// simply can't name `EngineEvent<T>` there.
pub trait EngineMessage: sealed::Sealed + Message + std::fmt::Debug {}

macro_rules! engine_message {
    ($($t:ty),* $(,)?) => { $(
        impl sealed::Sealed for $t {}
        impl EngineMessage for $t {}
    )* };
}
engine_message!(EngineStart, EpochStart, LimitTriggered, LogInfo, LogWarn);
impl<T: Send + Sync + 'static> sealed::Sealed for EngineImproved<T> {}
impl<T: Send + Sync + 'static> EngineMessage for EngineImproved<T> {}

impl<T: Send + Sync + 'static> sealed::Sealed for EpochCompleted<T> {}
impl<T: Send + Sync + 'static> EngineMessage for EpochCompleted<T> {}

impl<T: Send + Sync + 'static> sealed::Sealed for EngineStopped<T> {}
impl<T: Send + Sync + 'static> EngineMessage for EngineStopped<T> {}

#[derive(Clone, Debug)]
pub struct LogInfo(pub String);

#[derive(Clone, Debug)]
pub struct LogWarn(pub String);

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

#[derive(Clone)]
pub struct EngineImproved<T> {
    pub index: usize,
    pub best: T,
    pub score: Score,
}

impl<T> Debug for EngineImproved<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Improved(index={}, score={:?})", self.index, self.score)
    }
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

#[derive(Clone)]
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

impl<T> Debug for EpochCompleted<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EpochCompleted(index={}, score={:?}, objective={:?})",
            self.index, self.score, self.objective
        )
    }
}

#[derive(Clone)]
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

impl<T> Debug for EngineStopped<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EngineStopped(index={}, score={:?})",
            self.index, self.score
        )
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

impl<T> Debug for EngineEvent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineEvent::Started(_) => write!(f, "Started"),
            EngineEvent::Stopped(s) => write!(f, "Stopped(index={}, score={:?})", s.index, s.score),
            EngineEvent::EpochStarted(s) => write!(f, "EpochStarted(index={})", s.index),
            EngineEvent::EpochCompleted(s) => write!(
                f,
                "EpochCompleted(index={}, score={:?}, objective={:?})",
                s.index, s.score, s.objective
            ),
            EngineEvent::Improved(s) => {
                write!(f, "Improved(index={}, score={:?})", s.index, s.score)
            }
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
