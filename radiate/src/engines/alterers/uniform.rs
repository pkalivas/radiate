use super::Crossover;
use super::Mutate;
use crate::Chromosome;

pub struct UniformCrossover {
    rate: f32,
}

impl UniformCrossover {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<C: Chromosome> Crossover<C> for UniformCrossover {
    fn rate(&self) -> f32 {
        self.rate
    }
}

pub struct UniformMutator {
    pub rate: f32,
}

impl UniformMutator {
    pub fn new(rate: f32) -> Self {
        UniformMutator { rate }
    }
}

impl<C: Chromosome> Mutate<C> for UniformMutator {
    fn rate(&self) -> f32 {
        self.rate
    }
}
