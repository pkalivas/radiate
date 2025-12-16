use radiate_core::{
    AlterResult, BoundedGene, Chromosome, FloatGene, Gene, Mutate, Rate, random_provider,
};

/// The `GaussianMutator` is a simple mutator that adds a small amount of Gaussian noise to the gene.
///
/// This mutator is for use with any [Chromosome] which holds [FloatGene]s.
#[derive(Debug, Clone)]
pub struct GaussianMutator {
    rate: Rate,
}

impl GaussianMutator {
    /// Create a new instance of the `GaussianMutator` with the given rate.
    /// The rate must be between 0.0 and 1.0.
    pub fn new(rate: impl Into<Rate>) -> Self {
        let rate = rate.into();
        GaussianMutator { rate }
    }
}

impl<C: Chromosome<Gene = FloatGene>> Mutate<C> for GaussianMutator {
    fn rate(&self) -> Rate {
        self.rate.clone()
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, rate: f32) -> AlterResult {
        let mut count = 0;

        random_provider::with_rng(|rand| {
            for gene in chromosome.genes_mut() {
                if rand.bool(rate) {
                    let min = *gene.min() as f64;
                    let max = *gene.max() as f64;

                    let std_dev = (max - min) * 0.25;
                    let value = *gene.allele() as f64;

                    let gaussian = rand.gaussian(value, std_dev);
                    let allele = gaussian.clamp(min, max) as f32;

                    *gene.allele_mut() = allele;

                    count += 1;
                }
            }
        });

        AlterResult::from(count)
    }
}
