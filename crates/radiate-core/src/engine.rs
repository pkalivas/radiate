use crate::{
    Chromosome, Ecosystem, Front, MetricSet, Objective, Phenotype, Population, Problem, Score,
    Species, metric_names,
};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

pub trait EngineIter<P: Epoch> {
    fn iter(self) -> impl Iterator<Item = P>;
}

impl<E, P> EngineIter<P> for E
where
    E: Engine<P>,
    P: Epoch,
{
    fn iter(self) -> impl Iterator<Item = P> {
        EngineIterator {
            engine: self,
            phantom: std::marker::PhantomData,
        }
    }
}

pub struct EngineIterator<E, P>
where
    E: Engine<P>,
    P: Epoch,
{
    pub(crate) engine: E,
    pub(crate) phantom: std::marker::PhantomData<P>,
}

impl<P, E> Iterator for EngineIterator<E, P>
where
    P: Epoch,
    E: Engine<P>,
{
    type Item = P;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.engine.evolve())
    }
}

pub trait Engine<P: Epoch> {
    fn evolve(&mut self) -> P;
}

pub trait EngineExt<P: Epoch, E: Engine<P>> {
    fn run<F>(&mut self, limit: F) -> P
    where
        F: Fn(&P) -> bool,
        Self: Sized;
}

impl<P, E> EngineExt<P, E> for E
where
    P: Epoch,
    E: Engine<P>,
{
    fn run<F>(&mut self, limit: F) -> P
    where
        F: Fn(&P) -> bool,
        Self: Sized,
    {
        loop {
            let epoch = self.evolve();

            if limit(&epoch) {
                break epoch;
            }
        }
    }
}

pub trait Epoch {
    type Value;
    type Chromosome: Chromosome;

    fn value(&self) -> &Self::Value;
    fn ecosystem(&self) -> &Ecosystem<Self::Chromosome>;
    fn index(&self) -> usize;
    fn metrics(&self) -> &MetricSet;
    fn objective(&self) -> &Objective;

    fn population(&self) -> &Population<Self::Chromosome> {
        &self.ecosystem().population()
    }

    fn species(&self) -> Option<&[Species<Self::Chromosome>]> {
        self.ecosystem().species().map(|s| s.as_slice())
    }

    fn time(&self) -> Duration {
        self.metrics()
            .get(metric_names::TIME)
            .map(|m| m.time_statistic().map(|t| t.sum()))
            .flatten()
            .unwrap_or_default()
    }

    fn seconds(&self) -> f64 {
        self.time().as_secs_f64()
    }
}

pub trait EngineStep<C>: Send + Sync
where
    C: Chromosome,
{
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
            .split("<")
            .next()
            .unwrap_or(std::any::type_name::<Self>())
            .split("::")
            .last()
            .unwrap_or("Unknown Step")
    }

    fn execute(&mut self, generation: usize, metrics: &mut MetricSet, ecosystem: &mut Ecosystem<C>);
}

pub struct Context<C, T>
where
    C: Chromosome,
{
    pub ecosystem: Ecosystem<C>,
    pub best: T,
    pub index: usize,
    pub metrics: MetricSet,
    pub epoch_metrics: MetricSet,
    pub score: Option<Score>,
    pub front: Arc<RwLock<Front<Phenotype<C>>>>,
    pub objective: Objective,
    pub problem: Arc<dyn Problem<C, T>>,
}

impl<C, T> Clone for Context<C, T>
where
    C: Chromosome + Clone,
    T: Clone,
{
    fn clone(&self) -> Self {
        Context {
            ecosystem: self.ecosystem.clone(),
            best: self.best.clone(),
            index: self.index,
            metrics: self.metrics.clone(),
            epoch_metrics: self.epoch_metrics.clone(),
            score: self.score.clone(),
            front: self.front.clone(),
            objective: self.objective.clone(),
            problem: Arc::clone(&self.problem),
        }
    }
}
