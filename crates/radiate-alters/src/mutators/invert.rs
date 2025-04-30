use radiate_core::{AlterResult, Chromosome, Mutate, random_provider};

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
        if !(0.0..=1.0).contains(&rate) {
            panic!("rate must be between 0.0 and 1.0");
        }

        InversionMutator { rate }
    }
}

impl<C: Chromosome> Mutate<C> for InversionMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut mutations = 0;

        if random_provider::random::<f32>() < rate {
            let start = random_provider::range(0..chromosome.len());
            let end = random_provider::range(start..chromosome.len());

            chromosome.as_mut()[start..end].reverse();
            mutations += 1;
        }

        mutations.into()
    }
}
