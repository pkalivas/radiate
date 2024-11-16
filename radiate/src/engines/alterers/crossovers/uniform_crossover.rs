use crate::engines::alterers::crossovers::crossover::Crossover;
use crate::engines::genome::genes::gene::Gene;

pub struct UniformCrossover {
    pub rate: f32,
}

impl UniformCrossover {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<G: Gene<G, A>, A> Crossover<G, A> for UniformCrossover {
    fn cross_rate(&self) -> f32 {
        self.rate
    }
}
