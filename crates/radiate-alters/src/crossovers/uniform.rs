use radiate_core::{Chromosome, Crossover, Expr, Expr, ExprSet};

const UNIFORM_CROSSOVER_RATE: &str = "crossover.uniform.rate";

pub struct UniformCrossover {
    rate: Expr,
}

impl UniformCrossover {
    pub fn new(rate: impl Into<Expr>) -> Self {
        Self { rate: rate.into().alias(UNIFORM_CROSSOVER_RATE) }
    }
}

impl<C: Chromosome> Crossover<C> for UniformCrossover {
    fn rates(&self) -> ExprSet {
        ExprSet::from(self.rate.clone())
    }
}
