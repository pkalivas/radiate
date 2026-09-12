use radiate_core::{AlterContext, BitGene, Chromosome, Expr, Mutate, RateSet, random_provider};

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
    C: Chromosome<Gene = BitGene>,
{
    fn rates(&self) -> radiate_core::RateSet {
        RateSet::new(self.rate.clone())
    }

    fn mutate_chromosome(&mut self, chromosome: &mut C, ctx: &mut AlterContext) -> usize {
        let mut flips = 0;
        random_provider::with_rng(|rng| {
            for gene in chromosome.iter_mut() {
                if rng.bool(ctx.rate()) {
                    gene.flip();
                    flips += 1;
                }
            }
        });

        flips
    }
}
