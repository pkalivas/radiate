use radiate_core::{Chromosome, Crossover, Rate};

pub struct UniformCrossover {
    rate: Rate,
}

impl UniformCrossover {
    pub fn new(rate: impl Into<Rate>) -> Self {
        let rate = rate.into();
        Self { rate }
    }
}

impl<C: Chromosome> Crossover<C> for UniformCrossover {
    fn rate(&self) -> Rate {
        self.rate.clone()
    }
}
