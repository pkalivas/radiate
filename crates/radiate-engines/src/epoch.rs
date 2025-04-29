use crate::Chromosome;
use radiate_core::engine::EngineContext;
use radiate_core::{Ecosystem, Epoch, Front, MetricSet, Phenotype, Score};
use std::fmt::Debug;

pub struct Generation<C, T>
where
    C: Chromosome,
{
    pub ecosystem: Ecosystem<C>,
    pub best: T,
    pub index: usize,
    pub metrics: MetricSet,
    pub score: Score,
}

impl<C: Chromosome, T> Generation<C, T> {
    pub fn score(&self) -> &Score {
        &self.score
    }
}

impl<C: Chromosome, T> Epoch<C> for Generation<C, T> {
    type Result = T;

    fn ecosystem(&self) -> &Ecosystem<C> {
        &self.ecosystem
    }

    fn result(&self) -> &Self::Result {
        &self.best
    }

    fn index(&self) -> usize {
        self.index
    }

    fn metrics(&self) -> &MetricSet {
        &self.metrics
    }
}

impl<C: Chromosome, T: Clone> From<&EngineContext<C, T>> for Generation<C, T> {
    fn from(context: &EngineContext<C, T>) -> Self {
        Generation {
            ecosystem: context.ecosystem.clone(),
            best: context.best.clone(),
            index: context.index,
            metrics: context.metrics.clone(),
            score: context.score.clone().unwrap(),
        }
    }
}

impl<C, T: Debug> Debug for Generation<C, T>
where
    C: Chromosome,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EngineOutput {{\n")?;
        write!(f, "  best: {:?},\n", self.best)?;
        write!(f, "  score: {:?},\n", self.score)?;
        write!(f, "  index: {:?},\n", self.index)?;
        write!(f, "  size: {:?},\n", self.ecosystem.population.len())?;
        write!(f, "  duration: {:?},\n", self.time())?;
        write!(f, "  metrics: {:?},\n", self.metrics)?;
        write!(f, "}}")
    }
}

pub struct MultiObjectiveGeneration<C>
where
    C: Chromosome,
{
    pub ecosystem: Ecosystem<C>,
    pub front: Front<Phenotype<C>>,
    pub index: usize,
    pub metrics: MetricSet,
}

impl<C: Chromosome> Epoch<C> for MultiObjectiveGeneration<C>
where
    C: Chromosome,
{
    type Result = Front<Phenotype<C>>;

    fn ecosystem(&self) -> &Ecosystem<C> {
        &self.ecosystem
    }

    fn result(&self) -> &Self::Result {
        &self.front
    }

    fn index(&self) -> usize {
        self.index
    }

    fn metrics(&self) -> &MetricSet {
        &self.metrics
    }
}

impl<C: Chromosome, T: Clone> From<&EngineContext<C, T>> for MultiObjectiveGeneration<C> {
    fn from(context: &EngineContext<C, T>) -> Self {
        MultiObjectiveGeneration {
            ecosystem: context.ecosystem.clone(),
            front: context.front.read().unwrap().clone(),
            index: context.index,
            metrics: context.metrics.clone(),
        }
    }
}
