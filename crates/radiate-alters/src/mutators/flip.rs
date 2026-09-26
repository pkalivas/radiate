use radiate_core::{
    AlterContext, BitGene, ContiguousChromosome, Expr, Mutate, RateSet, random_provider,
};

/// Flips each [`BitGene`] in a chromosome independently with probability `rate`.
///
/// Genes are picked with [`random_provider::bernoulli_indices`], so at low rates
/// the cost scales with the number of flipped genes rather than the length of
/// the chromosome. A rate `<= 0` flips nothing and a rate `>= 1` flips every gene.
#[derive(Debug, Clone)]
pub struct BitFlipMutator {
    rate: Expr,
}

impl BitFlipMutator {
    pub fn new(rate: impl Into<Expr>) -> Self {
        Self { rate: rate.into() }
    }
}

impl<C> Mutate<C> for BitFlipMutator
where
    C: ContiguousChromosome<Gene = BitGene>,
{
    fn rates(&self) -> radiate_core::RateSet {
        RateSet::new(self.rate.clone())
    }

    #[inline]
    fn mutate_chromosome(&mut self, chromosome: &mut C, ctx: &mut AlterContext) -> usize {
        let p = ctx.rate();
        debug_assert!(p.is_finite());

        let genes = chromosome.as_mut_slice();

        let mut flips = 0;
        random_provider::with_rng(|rng| {
            rng.bernoulli_indices(p, 0..genes.len(), |i| {
                genes[i].flip();
                flips += 1;
            });
        });

        flips
    }
}
