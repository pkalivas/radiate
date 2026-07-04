use radiate_core::{
    AlterContext, AlterResult, BoundedGene, Chromosome, Expr, Expr, ExprSet, FloatGene, Gene,
    Mutate, SmallStr, random_provider,
};
use radiate_utils::{Float, Primitive};

const GAUSSIAN_MUTATOR_RATE: SmallStr = SmallStr::from_static("mutator.gaussian.rate");

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
        GaussianMutator { rate: rate.into().alias(GAUSSIAN_MUTATOR_RATE) }
    }
}

impl<F, C> Mutate<C> for GaussianMutator
where
    F: Float + Primitive,
    C: Chromosome<Gene = FloatGene<F>>,
{
    fn rates(&self) -> ExprSet {
        ExprSet::from(self.rate.clone())
    }

    #[inline]
    fn mutate_chromosome(&mut self, chromosome: &mut C, ctx: &mut AlterContext) -> AlterResult {
        let mut count = 0;

        random_provider::with_rng(|rand| {
            for gene in chromosome.as_mut_slice() {
                if rand.bool(ctx.rate()) {
                    // The reason we use the sampling min/max from the gene here instead of it's
                    // 'bounds' is because this operation is essentially a form of 'local search'
                    // and we want to ensure that the mutated value is not too far from the original value.
                    let min = gene.min().extract::<f64>().unwrap();
                    let max = gene.max().extract::<f64>().unwrap();

                    let std_dev = (max - min) * 0.25;
                    let value = gene.allele().extract::<f64>().unwrap();

                    let gaussian = rand.gaussian(value, std_dev);
                    let allele = gaussian.clamp(min, max);

                    *gene.allele_mut() = allele.extract::<F>().unwrap();

                    count += 1;
                }
            }
        });

        AlterResult::from(count)
    }
}
