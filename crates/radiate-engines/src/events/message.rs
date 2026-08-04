use crate::context::EvolutionContext;
use radiate_core::{
    ActorPanicked, ActorSubscribed, Chromosome, Ecosystem, Message, MetricSet, Objective, Score,
};
use std::{fmt::Debug, sync::Arc, time::Duration};

mod sealed {
    pub trait Sealed {}
}

pub trait EngineMessage: sealed::Sealed + Message + std::fmt::Debug {}

macro_rules! engine_message {
    ($($t:ty),* $(,)?) => { $(
        impl sealed::Sealed for $t {}
        impl EngineMessage for $t {}
    )* };
}
engine_message!(
    EngineStart,
    EpochStart,
    LimitTriggered,
    LimitProgress,
    Log,
    CheckpointSaved,
    ActorSubscribed,
    ActorPanicked
);
impl<T: Send + Sync + 'static> sealed::Sealed for Improvement<T> {}
impl<T: Send + Sync + 'static> EngineMessage for Improvement<T> {}

impl<T: Send + Sync + 'static> sealed::Sealed for EpochComplete<T> {}
impl<T: Send + Sync + 'static> EngineMessage for EpochComplete<T> {}

impl<T: Send + Sync + 'static> sealed::Sealed for EngineStop<T> {}
impl<T: Send + Sync + 'static> EngineMessage for EngineStop<T> {}

impl<C: Chromosome + Clone> sealed::Sealed for EcosystemSnapshot<C> {}
impl<C: Chromosome + Clone + Send + Sync + 'static> EngineMessage for EcosystemSnapshot<C> {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CheckpointSaved {
    pub index: usize,
    pub path: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LogLevel {
    Info,
    Warn,
}

#[derive(Clone, Debug)]
pub struct Log {
    pub level: LogLevel,
    pub index: Option<usize>,
    pub message: String,
}

impl Log {
    pub fn info<S: Into<String>>(index: Option<usize>, msg: S) -> Self {
        Log {
            level: LogLevel::Info,
            index,
            message: msg.into(),
        }
    }

    pub fn warn<S: Into<String>>(index: Option<usize>, msg: S) -> Self {
        Log {
            level: LogLevel::Warn,
            index,
            message: msg.into(),
        }
    }

    pub fn level(&self) -> LogLevel {
        self.level
    }

    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug)]
pub struct LimitTriggered {
    pub generation: usize,
    pub kind: &'static str,
    pub description: String,
}

impl LimitTriggered {
    pub fn new<S: Into<String>>(generation: usize, kind: &'static str, description: S) -> Self {
        LimitTriggered {
            generation,
            kind,
            description: description.into(),
        }
    }

    pub fn index(&self) -> usize {
        self.generation
    }

    pub fn kind(&self) -> &'static str {
        self.kind
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

#[derive(Clone, Debug)]
pub enum LimitProgress {
    Generations {
        current: usize,
        limit: usize,
    },
    Time {
        generation: usize,
        elapsed: Duration,
        limit: Duration,
    },
    Score {
        generation: usize,
        current: Score,
        limit: Score,
    },
    Convergence {
        generation: usize,
        window: usize,
        epsilon: f32,
        diff: f32,
    },
}

impl LimitProgress {
    pub fn generations(current: usize, limit: usize) -> Self {
        LimitProgress::Generations { current, limit }
    }

    pub fn time(generation: usize, elapsed: Duration, limit: Duration) -> Self {
        LimitProgress::Time {
            generation,
            elapsed,
            limit,
        }
    }

    pub fn score(generation: usize, current: Score, limit: Score) -> Self {
        LimitProgress::Score {
            generation,
            current,
            limit,
        }
    }

    pub fn convergence(generation: usize, window: usize, epsilon: f32, diff: f32) -> Self {
        LimitProgress::Convergence {
            generation,
            window,
            epsilon,
            diff,
        }
    }

    pub fn index(&self) -> usize {
        match self {
            LimitProgress::Generations { current, .. } => *current,
            LimitProgress::Time { generation, .. } => *generation,
            LimitProgress::Score { generation, .. } => *generation,
            LimitProgress::Convergence { generation, .. } => *generation,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            LimitProgress::Generations { .. } => "Generations",
            LimitProgress::Time { .. } => "Time",
            LimitProgress::Score { .. } => "Score",
            LimitProgress::Convergence { .. } => "Convergence",
        }
    }

    pub fn description(&self) -> String {
        match self {
            LimitProgress::Generations { current, limit } => {
                format!("[LIMIT] Generation progress: {current}/{limit}")
            }
            LimitProgress::Time {
                generation,
                elapsed,
                limit,
            } => {
                format!("[LIMIT] Time progress: {elapsed:?}/{limit:?} (Generation {generation})")
            }
            LimitProgress::Score {
                generation,
                current,
                limit,
            } => {
                format!("[LIMIT] Score progress: {current:?}/{limit:?} (Generation {generation})")
            }
            LimitProgress::Convergence {
                generation,
                window,
                epsilon,
                diff,
            } => format!(
                "[LIMIT] Convergence progress: |delta|={:.6} <= epsilon={epsilon} over window={window} (Generation {generation})",
                diff
            ),
        }
    }

    pub fn progress(&self) -> f32 {
        match self {
            LimitProgress::Generations { current, limit } => *current as f32 / *limit as f32,
            LimitProgress::Time {
                generation: _,
                elapsed,
                limit,
            } => elapsed.as_secs_f32() / limit.as_secs_f32(),
            LimitProgress::Score {
                generation: _,
                current,
                limit,
            } => {
                let mut total = 0.0;

                for (c, l) in current.iter().zip(limit.iter()) {
                    total += l - c;
                }
                if total == 0.0 { 0.0 } else { total }
            }
            LimitProgress::Convergence { diff, epsilon, .. } => {
                if *epsilon == 0.0 {
                    0.0
                } else {
                    diff / epsilon
                }
            }
        }
    }
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
