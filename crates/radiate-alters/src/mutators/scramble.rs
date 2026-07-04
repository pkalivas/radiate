use radiate_core::{AlterContext, AlterResult, Chromosome, Expr, ExprSet, Mutate, random_provider};

const SCRAMBLE_MUTATOR_RATE: &str = "mutator.scramble.rate";

/// The [ScrambleMutator] is a simple mutator that scrambles a random section of the [Chromosome].
///
/// Because the slice of the chromosome is of random length, with small chromosomes, the scrambling
/// may not be very effective. This mutator is best used with larger [Chromosome]s.
#[derive(Debug, Clone)]
pub struct ScrambleMutator {
    rate: Expr,
}

impl ScrambleMutator {
    pub fn new(rate: impl Into<Expr>) -> Self {
        ScrambleMutator { rate: rate.into() }
    }
}

impl<C: Chromosome> Mutate<C> for ScrambleMutator {
    fn expressions(&self) -> ExprSet {
        ExprSet::from(self.rate.clone().alias(SCRAMBLE_MUTATOR_RATE))
    }

    #[inline]
    fn mutate_chromosome(&mut self, chromosome: &mut C, ctx: &mut AlterContext) -> AlterResult {
        let mut mutations = 0;

        random_provider::with_rng(|rand| {
            if rand.bool(ctx.rate()) {
                let start = rand.range(0..chromosome.len());
                let end = rand.range(start..chromosome.len());
                let segment = &mut chromosome.as_mut_slice()[start..end];

                rand.shuffle(segment);
                mutations += 1;
            }
        });

        AlterResult::from(mutations)
    }
}
