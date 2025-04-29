use crate::{Chromosome, Pipeline};
use radiate_core::engine::EngineContext;
use radiate_core::timer::Timer;
use radiate_core::{Ecosystem, Engine, Epoch, Front, MetricSet, Phenotype, metric_names};

pub struct MultiObjectiveEngine<C, T>
where
    C: Chromosome + 'static,
    T: Clone + Send + 'static,
{
    context: EngineContext<C, T>,
    pipeline: Pipeline<C>,
}

impl<C, T> MultiObjectiveEngine<C, T>
where
    C: Chromosome,
    T: Clone + Send,
{
    pub fn new(context: EngineContext<C, T>, pipeline: Pipeline<C>) -> Self {
        MultiObjectiveEngine { context, pipeline }
    }
}

impl<C, T> Engine<C, T> for MultiObjectiveEngine<C, T>
where
    C: Chromosome,
    T: Clone + Send,
{
    type Epoch = MultiObjectiveGeneration<C>;

    fn run<F>(&mut self, limit: F) -> Self::Epoch
    where
        F: Fn(&MultiObjectiveGeneration<C>) -> bool,
    {
        loop {
            let epoch = self.next();

            if limit(&epoch) {
                break epoch;
            }
        }
    }

    fn next(&mut self) -> Self::Epoch {
        let timer = Timer::new();
        self.pipeline.run(
            self.context.index,
            &mut self.context.metrics,
            &mut self.context.ecosystem,
        );

        self.context
            .metrics
            .upsert_time(metric_names::EVOLUTION_TIME, timer.duration());

        self.context.index += 1;

        (&self.context).into()
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
            front: context.front.clone(),
            index: context.index,
            metrics: context.metrics.clone(),
        }
    }
}
