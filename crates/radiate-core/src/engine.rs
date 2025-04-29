use crate::{
    Chromosome, Ecosystem, Front, MetricSet, Objective, Phenotype, Population, Problem, Score,
    metric_names,
};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

pub trait Engine<C: Chromosome, T> {
    type Epoch: Epoch<C>;

    fn next(&mut self) -> Self::Epoch;

    fn run<F>(&mut self, limit: F) -> Self::Epoch
    where
        F: Fn(&Self::Epoch) -> bool,
    {
        loop {
            let epoch = self.next();

            if limit(&epoch) {
                break epoch;
            }
        }
    }
}

pub trait Epoch<C: Chromosome> {
    type Result;

    fn result(&self) -> &Self::Result;
    fn ecosystem(&self) -> &Ecosystem<C>;
    fn index(&self) -> usize;
    fn metrics(&self) -> &MetricSet;

    fn population(&self) -> &Population<C> {
        &self.ecosystem().population
    }

    fn time(&self) -> Duration {
        self.metrics()
            .get(metric_names::EVOLUTION_TIME)
            .unwrap()
            .time_sum()
            .unwrap()
    }

    fn seconds(&self) -> f64 {
        self.time().as_secs_f64()
    }
}

pub trait EngineStep<C>
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

pub struct EngineContext<C, T>
where
    C: Chromosome,
{
    pub ecosystem: Ecosystem<C>,
    pub best: T,
    pub index: usize,
    pub metrics: MetricSet,
    pub score: Option<Score>,
    pub front: Arc<RwLock<Front<Phenotype<C>>>>,
    pub objective: Objective,
    pub problem: Arc<dyn Problem<C, T>>,
}

impl<C, T> Clone for EngineContext<C, T>
where
    C: Chromosome,
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            ecosystem: self.ecosystem.clone(),
            best: self.best.clone(),
            index: self.index,
            metrics: self.metrics.clone(),
            score: self.score.clone(),
            front: self.front.clone(),
            objective: self.objective.clone(),
            problem: Arc::clone(&self.problem),
        }
    }
}
