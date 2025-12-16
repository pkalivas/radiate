use radiate_core::{AlterResult, Chromosome, Mutate, Rate, random_provider};

/// The [ScrambleMutator] is a simple mutator that scrambles a random section of the [Chromosome].
///
/// Because the slice of the chromosome is of random length, with small chromosomes, the scrambling
/// may not be very effective. This mutator is best used with larger [Chromosome]s.
#[derive(Debug, Clone)]
pub struct ScrambleMutator {
    rate: Rate,
}

impl ScrambleMutator {
    pub fn new(rate: impl Into<Rate>) -> Self {
        let rate = rate.into();
        ScrambleMutator { rate }
    }
}

impl<C: Chromosome> Mutate<C> for ScrambleMutator {
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
                let segment = &mut chromosome.genes_mut()[start..end];

                rand.shuffle(segment);
                mutations += 1;
            }
        });

        AlterResult::from(mutations)
    }
}
