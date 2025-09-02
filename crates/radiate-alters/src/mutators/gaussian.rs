use crate::{
    Expr,
    dsl::{gausian_mutate as mg, *},
};
use radiate_core::{
    BoundedGene, Chromosome, FloatGene, Gene, Mutate,
    chromosomes::gene::{HasNumericSlot, apply_numeric_slot_mut},
    random_provider,
};

/// The `GaussianMutator` is a simple mutator that adds a small amount of Gaussian noise to the gene.
///
/// This mutator is for use with the `FloatChromosome` or any `Chromosome` which holds `FloatGene`s.
pub struct GaussianMutator {
    rate: f32,
    expr: Expr<FloatGene>,
}

impl GaussianMutator {
    /// Create a new instance of the `GaussianMutator` with the given rate.
    /// The rate must be between 0.0 and 1.0.
    pub fn new(rate: f32) -> Self {
        if !(0.0..=1.0).contains(&rate) {
            panic!("Rate must be between 0 and 1");
        }

        let expr: Expr<FloatGene> = all(prob(rate, gaussian_by_bounds_generic()));
        println!("GaussianMutator expr: {:?}", expr.dump_tree());

        GaussianMutator { rate, expr }
    }
}

impl<C: Chromosome<Gene = FloatGene>> Mutate<C> for GaussianMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn mutate_chromosome(&self, chromosome: &mut C, _: f32) -> radiate_core::AlterResult {
        self.expr.apply_slice(chromosome.genes_mut()).into()
    }

    #[inline]
    fn mutate_gene(&self, gene: &C::Gene) -> C::Gene {
        let min = *gene.min() as f64;
        let max = *gene.max() as f64;

        let std_dev = (max - min) * 0.25;
        let value = *gene.allele() as f64;

        let gaussian = random_provider::gaussian(value, std_dev);
        let allele = gaussian.clamp(min, max) as f32;

        gene.with_allele(&allele)
    }
}
