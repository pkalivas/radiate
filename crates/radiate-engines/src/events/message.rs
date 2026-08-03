use crate::context::EvolutionContext;
use radiate_core::{Chromosome, Envelope, MetricSet, Objective, Score};
use std::fmt::Debug;

/// Internal, borrowed carrier used only inside `GeneticEngine::step()`/
/// `Drop` — cheap to construct since it doesn't clone anything out of the
/// context. `EventBus::publish` is what turns this into real message
/// payloads, and only for the specific kinds anyone's actually subscribed
/// to.
pub enum EngineMessage<'a, C, T>
where
    C: Chromosome,
    T: Clone,
{
    Start(&'a EvolutionContext<C, T>),
    Stop(&'a EvolutionContext<C, T>),
    EpochStart(&'a EvolutionContext<C, T>),
    EpochEnd(&'a EvolutionContext<C, T>),
    Improvement(&'a EvolutionContext<C, T>),
}

pub struct StartedData;
pub type Started = Envelope<StartedData>;

pub struct StoppedData<T> {
    pub index: usize,
    pub best: T,
    pub metrics: MetricSet,
    pub score: Score,
}
pub type Stopped<T> = Envelope<StoppedData<T>>;

pub struct EpochStartedData {
    pub index: usize,
}
pub type EpochStarted = Envelope<EpochStartedData>;

pub struct EpochCompletedData<T> {
    pub index: usize,
    pub best: T,
    pub metrics: MetricSet,
    pub score: Score,
    pub objective: Objective,
}
pub type EpochCompleted<T> = Envelope<EpochCompletedData<T>>;

pub struct ImprovedData<T> {
    pub index: usize,
    pub best: T,
    pub score: Score,
}
pub type Improved<T> = Envelope<ImprovedData<T>>;

/// The "give me every kind of engine event" umbrella — one type, so a single
/// wildcard subscription still works, at the cost of a second `send` (gated
/// by its own `has_subscribers` check, same as the specific-kind path) when
/// both a wildcard and a specific-kind subscriber exist for the same event.
#[derive(Clone)]
pub enum EngineEvent<T> {
    Started(Started),
    Stopped(Stopped<T>),
    EpochStarted(EpochStarted),
    EpochCompleted(EpochCompleted<T>),
    Improved(Improved<T>),
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
        }
    }
}
