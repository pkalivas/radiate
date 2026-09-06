use radiate_core::{
    AlterContext, BoundedGene, Chromosome, Expr, FloatGene, Gene, Mutate, RateSet, random_provider,
};
use radiate_utils::Float;

/// The `GaussianMutator` is a simple mutator that adds a small amount of Gaussian noise to the gene.
///
/// This mutator is for use with any [Chromosome] which holds [FloatGene]s.
#[derive(Debug, Clone)]
pub struct GaussianMutator {
    rate: Expr,
}

impl GaussianMutator {
    /// Create a new instance of the `GaussianMutator` with the given rate.
    /// The rate must be between 0.0 and 1.0.
    pub fn new(rate: impl Into<Expr>) -> Self {
        GaussianMutator { rate: rate.into() }
    }
}

impl<F, C> Mutate<C> for GaussianMutator
where
    F: Float,
    C: Chromosome<Gene = FloatGene<F>>,
{
    fn rates(&self) -> RateSet {
        RateSet::new(self.rate.clone())
    }

    #[inline]
    fn mutate_chromosome(&mut self, chromosome: &mut C, ctx: &mut AlterContext) -> usize {
        let mut count = 0;

        random_provider::with_rng(|rand| {
            for gene in chromosome.iter_mut() {
                if rand.bool(ctx.rate()) {
                    // The reason we use the sampling min/max from the gene here instead of it's
                    // 'bounds' is because this operation is essentially a form of 'local search'
                    // and we want to ensure that the mutated value is not too far from the original value.
                    let min = gene.init_min();
                    let max = gene.init_max();

                    let std_dev = (*max - *min) * F::from(0.25).unwrap();
                    let gaussian = rand.gaussian(*gene.allele(), std_dev);

                    gene.set_allele(gaussian.clamp(*min, *max));

                    count += 1;
                }
            }
        });

        count
    }
}
