use radiate_core::{Chromosome, Crossover, Rate, Valid};

pub struct UniformCrossover {
    rate: Rate,
}

impl UniformCrossover {
    pub fn new(rate: impl Into<Rate>) -> Self {
        let rate = rate.into();
        if !rate.is_valid() {
            panic!("Rate {rate:?} is not valid. Must be between 0.0 and 1.0",);
        }

        Self { rate }
    }
}

impl<C: Chromosome> Crossover<C> for UniformCrossover {
    fn rate(&self) -> Rate {
        self.rate.clone()
    }
}
