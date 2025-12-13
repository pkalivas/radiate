use crate::context::Context;
use radiate_core::{Chromosome, MetricSet, Objective, Score};
use std::fmt::Debug;

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

pub enum EngineEvent<T> {
    Start,
    Stop(usize, T, MetricSet, Score),
    EpochStart(usize),
    EpochComplete(usize, T, MetricSet, Score, Objective),
    Improvement(usize, T, Score),
}

impl<T: Debug> Debug for EngineEvent<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineEvent::Start => write!(f, "EngineEvent::Start"),
            EngineEvent::Stop(_, _, metrics, best_score) => write!(
                f,
                "EngineEvent::Stop {{ metrics: {:?}, best_score: {:?} }}",
                metrics, best_score
            ),
            EngineEvent::EpochStart(epoch) => {
                write!(f, "EngineEvent::EpochStart {{ epoch: {} }}", epoch)
            }
            EngineEvent::EpochComplete(epoch, _, metrics, best_score, objective) => write!(
                f,
                "EngineEvent::EpochComplete {{ epoch: {}, metrics: {:?}, best_score: {:?}, objective: {:?} }}",
                epoch, metrics, best_score, objective
            ),
            EngineEvent::Improvement(epoch, _, best_score) => write!(
                f,
                "EngineEvent::Improvement {{ epoch: {}, best_score: {:?} }}",
                epoch, best_score
            ),
        }
    }
}
