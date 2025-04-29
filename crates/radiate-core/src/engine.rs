use crate::{Chromosome, Ecosystem, MetricSet, Population, Score};

pub trait Engine<C: Chromosome, T> {
    type Epoch: Epoch<C, T>;

    fn next(&mut self);

    fn run<F>(&mut self, limit: F) -> Self::Epoch
    where
        F: Fn(&Self::Epoch) -> bool;
}

pub trait Epoch<C: Chromosome, T> {
    fn population(&self) -> &Population<C>;
    fn generation(&self) -> usize;
    fn score(&self) -> &Score;
    fn best(&self) -> &T;
    fn metrics(&self) -> &MetricSet;
}

pub trait EngineStep<C: Chromosome, T> {
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
