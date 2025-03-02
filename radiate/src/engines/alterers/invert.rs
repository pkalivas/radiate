use super::{AlterAction, AlterResult, Alterer, IntoAlter, Mutate};
use crate::{Chromosome, random_provider};

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

impl<C: Chromosome> Mutate<C> for InversionMutator {
    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut mutations = 0;

        if random_provider::random::<f32>() < rate {
            let start = random_provider::random_range(0..chromosome.len());
            let end = random_provider::random_range(start..chromosome.len());

            chromosome.as_mut()[start..end].reverse();
            mutations += 1;
        }

        mutations.into()
    }
}

impl<C: Chromosome> IntoAlter<C> for InversionMutator {
    fn into_alter(self) -> Alterer<C> {
        Alterer::new(
            "InversionMutator",
            self.rate,
            AlterAction::Mutate(Box::new(self)),
        )
    }
}
