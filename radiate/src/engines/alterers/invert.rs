use crate::{random_provider, Chromosome, EngineCompoment};

use super::{Alter, AlterAction, Mutate};

/// The `InversionMutator` is a simple mutator that inverts a random section of the chromosome.
///
/// Because the slice of the chromosome is of random length, with small chromosomes, the inversion
/// may not be very effective. This mutator is best used with larger chromosomes.
pub struct InversionMutator {
    rate: f32,
}

impl InversionMutator {
    /// Create a new instance of the `InversionMutator` with the given rate.
    /// The rate must be between 0.0 and 1.0.
    pub fn new(rate: f32) -> Self {
        if rate < 0.0 || rate > 1.0 {
            panic!("rate must be between 0.0 and 1.0");
        }

        InversionMutator { rate }
    }
}

impl EngineCompoment for InversionMutator {
    fn name(&self) -> &'static str {
        "InversionMutator"
    }
}

impl<C: Chromosome> Alter<C> for InversionMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<C: Chromosome> Mutate<C> for InversionMutator {
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C) -> i32 {
        let mut mutations = 0;

        if random_provider::random::<f32>() < self.rate {
            let start = random_provider::gen_range(0..chromosome.len());
            let end = random_provider::gen_range(start..chromosome.len());

            chromosome.as_mut()[start..end].reverse();
            mutations += 1;
        }

        mutations
    }
}
