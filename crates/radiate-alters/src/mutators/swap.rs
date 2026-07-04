use radiate_core::{AlterContext, AlterResult, Chromosome, Expr, Expr, ExprSet, Mutate, random_provider};

const SWAP_MUTATOR_RATE: &str = "mutator.swap.rate";

/// The [SwapMutator] is a simple mutator that swaps random genes in the [Chromosome].
#[derive(Debug, Clone)]
pub struct SwapMutator {
    rate: Expr,
}

impl SwapMutator {
    pub fn new(rate: impl Into<Expr>) -> Self {
        SwapMutator { rate: rate.into().alias(SWAP_MUTATOR_RATE) }
    }
}

impl<C: Chromosome> Mutate<C> for SwapMutator {
    fn rates(&self) -> ExprSet {
        ExprSet::from(self.rate.clone())
    }

    #[inline]
    fn mutate_chromosome(&mut self, chromosome: &mut C, ctx: &mut AlterContext) -> AlterResult {
        let mut mutations = 0;

        random_provider::with_rng(|rand| {
            for i in 0..chromosome.len() {
                if rand.bool(ctx.rate()) {
                    let swap_index = rand.range(0..chromosome.len());
                    if swap_index == i {
                        continue;
                    }

                    chromosome.as_mut_slice().swap(i, swap_index);
                    mutations += 1;
                }
            }
        });

        AlterResult::from(mutations)
    }
}
