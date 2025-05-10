use crate::{
    Chromosome, Ecosystem, Front, MetricSet, Objective, Phenotype, Population, Problem, Score,
    Species, metric_names,
};
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

pub trait Engine {
    type Chromosome: Chromosome;
    type Epoch: Epoch<Chromosome = Self::Chromosome>;

    fn next(&mut self) -> Self::Epoch;
}

pub trait EngineExt<E: Engine> {
    fn run<F>(&mut self, limit: F) -> E::Epoch
    where
        F: Fn(&E::Epoch) -> bool,
        Self: Sized;
}

impl<E> EngineExt<E> for E
where
    E: Engine,
{
    fn run<F>(&mut self, limit: F) -> E::Epoch
    where
        F: Fn(&E::Epoch) -> bool,
        Self: Sized,
    {
        loop {
            let epoch = self.next();

            if limit(&epoch) {
                break epoch;
            }
        }
    }
}

pub trait Epoch {
    type Chromosome: Chromosome;
    type Value;

    fn value(&self) -> &Self::Value;
    fn ecosystem(&self) -> &Ecosystem<Self::Chromosome>;
    fn index(&self) -> usize;
    fn metrics(&self) -> &MetricSet;
    fn objective(&self) -> &Objective;

    fn population(&self) -> &Population<Self::Chromosome> {
        &self.ecosystem().population()
    }

    fn species(&self) -> Option<&Vec<Species<Self::Chromosome>>> {
        self.ecosystem().species()
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
    pub score: Option<Score>,
    pub front: Arc<RwLock<Front<Phenotype<C>>>>,
    pub objective: Objective,
    pub problem: Arc<dyn Problem<C, T>>,
}

impl<C, T> Clone for Context<C, T>
where
    C: Chromosome,
    T: Clone,
{
    fn clone(&self) -> Self {
        Context {
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
