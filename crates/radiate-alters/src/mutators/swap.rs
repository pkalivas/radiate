use radiate_core::{
    AlterContext, Expr, Mutate, RateSet, chromosomes::ContiguousChromosome, random_provider,
};

/// The [SwapMutator] is a simple mutator that swaps random genes in the [Chromosome].
#[derive(Debug, Clone)]
pub struct SwapMutator {
    rate: Expr,
}

impl SwapMutator {
    pub fn new(rate: impl Into<Expr>) -> Self {
        SwapMutator { rate: rate.into() }
    }
}

impl<C: ContiguousChromosome> Mutate<C> for SwapMutator {
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    #[inline]
    fn mutate_chromosome(&mut self, chromosome: &mut C, ctx: &mut AlterContext) -> usize {
        let mut mutations = 0;

        random_provider::bernoulli_indices(ctx.rate(), chromosome.len(), |index| {
            let swap_index = random_provider::range(0..chromosome.len());
            if swap_index == index {
                return;
            }

            chromosome.swap(index, swap_index);
            mutations += 1;
        });

        mutations
    }
}
