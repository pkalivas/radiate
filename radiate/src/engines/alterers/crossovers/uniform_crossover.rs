use crate::{Alterer, Chromosome, Crossover};

pub struct UniformCrossover {
    pub rate: f32,
}

impl UniformCrossover {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<C: Chromosome> Crossover<C> for UniformCrossover {
    fn cross_rate(&self) -> f32 {
        self.rate
    }

    fn name(&self) -> &'static str {
        "UniformCrossover"
    }
}
