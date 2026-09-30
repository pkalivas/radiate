use radiate_core::{
    AlterContext, BitChromosome, ContiguousChromosome, Expr, Gene, Mutate, PackedBitChromosome,
    RateSet, random_provider,
};

/// Flips each bit of a [`BitChromosome`] or [`PackedBitChromosome`] independently
/// with probability `rate`.
///
/// The rate is **per bit**, so a chromosome of `n` bits sees `n * rate` flips per
/// generation on average. A common starting point is `rate = 1 / n` - one expected
/// flip per chromosome.
///
/// Bits are picked with [`random_provider::bernoulli_indices`], so at low rates
/// the cost scales with the number of flipped bits rather than the length of
/// the chromosome. A rate `<= 0` or `NaN` flips nothing and a rate `>= 1` flips every bit.
///
/// On a [`PackedBitChromosome`] the rate is still per bit over
/// [`num_bits`](PackedBitChromosome::num_bits), each flip is a single XOR into its
/// word, and the returned count is in bits.
///
/// Implemented for [`BitChromosome`] and [`PackedBitChromosome`] specifically, not
/// for any chromosome whose gene is a [`BitGene`](radiate_core::BitGene).
#[derive(Debug, Clone)]
pub struct BitFlipMutator {
    rate: Expr,
}

impl BitFlipMutator {
    pub fn new(rate: impl Into<Expr>) -> Self {
        Self { rate: rate.into() }
    }
}

impl Mutate<BitChromosome> for BitFlipMutator {
    fn rates(&self) -> radiate_core::RateSet {
        RateSet::new(self.rate.clone())
    }

    #[inline]
    fn mutate_chromosome(
        &mut self,
        chromosome: &mut BitChromosome,
        ctx: &mut AlterContext,
    ) -> usize {
        let p = ctx.rate();
        let genes = chromosome.as_mut_slice();

        let mut flips = 0;
        random_provider::with_rng(|rng| {
            rng.bernoulli_indices(p, genes.len(), |i| {
                genes[i].flip();
                flips += 1;
            });
        });

        flips
    }
}

impl Mutate<PackedBitChromosome> for BitFlipMutator {
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    #[inline]
    fn mutate_chromosome(
        &mut self,
        chromosome: &mut PackedBitChromosome,
        ctx: &mut AlterContext,
    ) -> usize {
        let p = ctx.rate();
        let num_bits = chromosome.num_bits();
        let words = chromosome.as_mut_slice();

        let mut flips = 0;
        random_provider::with_rng(|rng| {
            rng.bernoulli_indices(p, num_bits, |k| {
                *words[k >> 6].allele_mut() ^= 1 << (k & 63);
                flips += 1;
            });
        });

        flips
    }
}
