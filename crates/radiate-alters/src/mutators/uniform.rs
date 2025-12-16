use radiate_core::{Chromosome, Mutate, Rate};

/// The [UniformMutator] is a simple mutator that applies uniform mutation to genes in a [Chromosome].
///
/// This mutator is essentially the 'default' mutator and is a good starting point for most problems.
#[derive(Debug, Clone)]
pub struct UniformMutator {
    pub rate: Rate,
}

impl UniformMutator {
    pub fn new(rate: impl Into<Rate>) -> Self {
        let rate = rate.into();
        UniformMutator { rate }
    }
}

impl<C: Chromosome> Mutate<C> for UniformMutator {
    fn rate(&self) -> Rate {
        self.rate.clone()
    }
}
