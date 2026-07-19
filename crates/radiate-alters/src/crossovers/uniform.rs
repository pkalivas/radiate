use radiate_core::{Chromosome, Crossover, Expr, RateSet};

pub struct UniformCrossover {
    rate: Expr,
}

impl UniformCrossover {
    pub fn new(rate: impl Into<Expr>) -> Self {
        Self { rate: rate.into() }
    }
}

impl<C: Chromosome> Crossover<C> for UniformCrossover {
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }
}
