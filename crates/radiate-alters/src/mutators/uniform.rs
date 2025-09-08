use radiate_core::{Chromosome, Mutate};

/// The [UniformMutator] is a simple mutator that applies uniform mutation to genes in a [Chromosome].
///
/// This mutator is essentially the 'default' mutator and is a good starting point for most problems.
#[derive(Debug, Clone)]
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
