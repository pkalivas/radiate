use radiate_core::{AlterResult, Chromosome, Mutate, Rate, random_provider};

/// The [InversionMutator] is a simple mutator that inverts a random section of the chromosome.
///
/// Because the slice of the chromosome is of random length, with small chromosomes, the inversion
/// may not be very effective. This mutator is best used with larger chromosomes.
#[derive(Debug, Clone)]
pub struct InversionMutator {
    rate: Rate,
}

impl InversionMutator {
    /// Create a new instance of the [InversionMutator] with the given rate.
    /// The rate must be between 0.0 and 1.0.
    pub fn new(rate: impl Into<Rate>) -> Self {
        let rate = rate.into();
        InversionMutator { rate }
    }
}

impl<C: Chromosome> Mutate<C> for InversionMutator {
    fn rate(&self) -> Rate {
        self.rate.clone()
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut mutations = 0;

        random_provider::with_rng(|rand| {
            if rand.bool(rate) {
                let start = rand.range(0..chromosome.len());
                let end = rand.range(start..chromosome.len());

                chromosome.genes_mut()[start..end].reverse();
                mutations += 1;
            }
        });

        AlterResult::from(mutations)
    }
}
