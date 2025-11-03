use crate::Context;
use radiate_core::{Chromosome, MetricSet, Score};

pub enum EngineMessage<'a, C, T>
where
    C: Chromosome,
{
    Start,
    Stop(&'a Context<C, T>),
    EpochStart(&'a Context<C, T>),
    EpochEnd(&'a Context<C, T>),
    StepStart(&'a Context<C, T>, &'static str),
    StepComplete(&'a Context<C, T>, &'static str),
    Improvement(&'a Context<C, T>),
}

pub enum EngineEvent<T> {
    Start,
    Stop(T, MetricSet, Score),
    EpochStart(usize),
    EpochComplete(usize, T, MetricSet, Score),
    StepStart(&'static str),
    StepComplete(&'static str),
    Improvement(usize, T, Score),
}
