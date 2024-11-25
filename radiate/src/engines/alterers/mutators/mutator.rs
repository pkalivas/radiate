use crate::Chromosome;

use super::mutate::Mutate;

pub struct Mutator {
    pub rate: f32,
}

impl Mutator {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<C: Chromosome> Mutate<C> for Mutator {
    fn mutate_rate(&self) -> f32 {
        self.rate
    }

    fn name(&self) -> &'static str {
        "Mutator"
    }
}
