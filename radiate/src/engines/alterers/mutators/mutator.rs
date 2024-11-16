use crate::engines::genome::genes::gene::Gene;

use super::mutate::Mutate;

pub struct Mutator {
    pub rate: f32,
}

impl Mutator {
    pub fn new(rate: f32) -> Self {
        Self { rate }
    }
}

impl<G: Gene<G, A>, A> Mutate<G, A> for Mutator {
    fn mutate_rate(&self) -> f32 {
        self.rate
    }
}
