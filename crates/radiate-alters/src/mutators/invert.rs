use radiate_core::{
    AlterContext, AlterResult, Chromosome, Expr, ExprSet, Mutate, SmallStr, random_provider,
};

const INVERSION_MUTATOR_RATE: SmallStr = SmallStr::from_static("mutator.inversion.rate");

/// The [InversionMutator] is a simple mutator that inverts a random section of the chromosome.
///
/// Because the slice of the chromosome is of random length, with small chromosomes, the inversion
/// may not be very effective. This mutator is best used with larger chromosomes.
#[derive(Debug, Clone)]
pub struct InversionMutator {
    rate: Expr,
}

impl InversionMutator {
    /// Create a new instance of the [InversionMutator] with the given rate.
    /// The rate must be between 0.0 and 1.0.
    pub fn new(rate: impl Into<Expr>) -> Self {
        let rate = rate.into();
        InversionMutator { rate }
    }
}

impl<C: Chromosome> Mutate<C> for InversionMutator {
    fn expressions(&self) -> ExprSet {
        ExprSet::from(self.rate.clone().alias(INVERSION_MUTATOR_RATE))
    }

    #[inline]
    fn mutate_chromosome(&mut self, chromosome: &mut C, ctx: &mut AlterContext) -> AlterResult {
        let mut mutations = 0;

        random_provider::with_rng(|rand| {
            if rand.bool(ctx.rate()) {
                let start = rand.range(0..chromosome.len());
                let end = rand.range(start..chromosome.len());

                chromosome.as_mut_slice()[start..end].reverse();
                mutations += 1;
            }
        });

        AlterResult::from(mutations)
    }
}
