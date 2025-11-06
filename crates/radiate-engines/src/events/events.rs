use crate::context::Context;
use radiate_core::{Chromosome, MetricSet, Score};

pub enum EngineMessage<'a, C, T>
where
    C: Chromosome,
{
    Start,
    Stop(&'a Context<C, T>),
    EpochStart(&'a Context<C, T>),
    EpochEnd(&'a Context<C, T>),
    Improvement(&'a Context<C, T>),
}

pub enum EngineEvent<T> {
    Start,
    Stop(T, MetricSet, Score),
    EpochStart(usize),
    EpochComplete(usize, T, MetricSet, Score),
    Improvement(usize, T, Score),
}
