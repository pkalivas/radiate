use radiate_core::{Chromosome, Mutate};

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
