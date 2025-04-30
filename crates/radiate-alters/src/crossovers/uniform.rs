use radiate_core::{Chromosome, Crossover};

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
