use crate::{Chromosome, MetricSet, Population, Score};

pub trait Epoch<C: Chromosome, T> {
    fn population(&self) -> &Population<C>;
    fn generation(&self) -> usize;
    fn score(&self) -> &Score;
    fn best(&self) -> &T;
    fn metrics(&self) -> &MetricSet;
}

pub trait Engine<C: Chromosome, T> {
    type Epoch: Epoch<C, T>;

    fn next(&mut self);

    fn run<F>(&mut self, limit: F) -> Self::Epoch
    where
        F: Fn(&Self::Epoch) -> bool;
}
