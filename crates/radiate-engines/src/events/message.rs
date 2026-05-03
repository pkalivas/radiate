use crate::context::Context;
use radiate_core::{Chromosome, MetricSet, Objective, Score};
use std::{fmt::Debug, sync::Arc};

pub enum EngineMessage<'a, C, T>
where
    C: Chromosome,
    T: Clone,
{
    Start,
    Stop(&'a Context<C, T>),
    EpochStart(&'a Context<C, T>),
    EpochEnd(&'a Context<C, T>),
    Improvement(&'a Context<C, T>),
}

pub enum EngineEventInner<T> {
    Start,
    Stop(usize, T, MetricSet, Score),
    EpochStart(usize),
    EpochComplete(usize, T, MetricSet, Score, Objective),
    Improvement(usize, T, Score),
}

impl<T: Debug> Debug for EngineEventInner<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineEventInner::Start => write!(f, "Start"),
            EngineEventInner::Stop(index, best, metrics, score) => write!(
                f,
                "Stop(index={}, best={:?}, metrics={:?}, score={:?})",
                index, best, metrics, score
            ),
            EngineEventInner::EpochStart(index) => write!(f, "EpochStart(index={})", index),
            EngineEventInner::EpochComplete(index, best, metrics, score, objective) => write!(
                f,
                "EpochComplete(index={}, best={:?}, metrics={:?}, score={:?}, objective={:?})",
                index, best, metrics, score, objective
            ),
            EngineEventInner::Improvement(index, best, score) => write!(
                f,
                "Improvement(index={}, best={:?}, score={:?})",
                index, best, score
            ),
        }
    }
}

pub struct EngineEvent<T> {
    inner: Arc<EngineEventInner<T>>,
}

impl<T> EngineEvent<T> {
    pub fn new(inner: EngineEventInner<T>) -> Self {
        EngineEvent {
            inner: Arc::new(inner),
        }
    }

    pub fn inner(&self) -> &EngineEventInner<T> {
        self.inner.as_ref()
    }

    pub fn is_start(&self) -> bool {
        matches!(self.inner(), EngineEventInner::Start)
    }

    pub fn is_stop(&self) -> bool {
        matches!(self.inner(), EngineEventInner::Stop(..))
    }

    pub fn is_epoch_start(&self) -> bool {
        matches!(self.inner(), EngineEventInner::EpochStart(..))
    }

    pub fn is_epoch_complete(&self) -> bool {
        matches!(self.inner(), EngineEventInner::EpochComplete(..))
    }

    pub fn is_improvement(&self) -> bool {
        matches!(self.inner(), EngineEventInner::Improvement(..))
    }
}

impl<T> Clone for EngineEvent<T> {
    fn clone(&self) -> Self {
        EngineEvent {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T: Debug> Debug for EngineEvent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EngineEvent::{:?}", self.inner())
    }
}
