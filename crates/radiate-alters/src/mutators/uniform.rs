use radiate_core::{Chromosome, Expr, Mutate, RateSet};

/// The [UniformMutator] is a simple mutator that applies uniform mutation to genes in a [Chromosome].
///
/// This mutator is essentially the 'default' mutator and is a good starting point for most problems.
#[derive(Debug, Clone)]
pub struct UniformMutator {
    pub rate: Expr,
}

impl UniformMutator {
    pub fn new(rate: impl Into<Expr>) -> Self {
        UniformMutator { rate: rate.into() }
    }
}

impl<C: Chromosome> Mutate<C> for UniformMutator {
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }
}
