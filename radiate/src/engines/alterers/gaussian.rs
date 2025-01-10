use crate::{random_provider, Chromosome, FloatGene, Gene, NumericGene};

use super::{Alter, AlterAction, EngineCompoment, Mutate};

/// The `GaussianMutator` is a simple mutator that adds a small amount of Gaussian noise to the gene.
///
/// This mutator is for use with the `FloatChromosome` or any `Chromosome` which holds `FloatGene`s.
pub struct GaussianMutator {
    rate: f32,
}

impl GaussianMutator {
    /// Create a new instance of the `GaussianMutator` with the given rate.
    /// The rate must be between 0.0 and 1.0.
    pub fn new(rate: f32) -> Self {
        if rate < 0.0 || rate > 1.0 {
            panic!("Rate must be between 0 and 1");
        }

        GaussianMutator { rate }
    }

    /// Clamp a value between a minimum and maximum value.
    /// If the value is less than the minimum, return the minimum. Else if the value is
    /// greater than the maximum, return the maximum. Without this function, the Gaussian noise
    /// could potentially generate a value outside of the gene's bounds or a value just plain unusable
    /// (e.g. NaN, +/- Inf).
    pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }
}

impl EngineCompoment for GaussianMutator {
    fn name(&self) -> &'static str {
        "GaussianMutator"
    }
}

impl<C: Chromosome<Gene = FloatGene>> Alter<C> for GaussianMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<C: Chromosome<Gene = FloatGene>> Mutate<C> for GaussianMutator {
    #[inline]
    fn mutate_gene(&self, gene: &C::Gene) -> C::Gene {
        let min = *gene.min() as f64;
        let max = *gene.max() as f64;

        let std_dev = (max - min) * 0.25;
        let value = *gene.allele() as f64;

        let gaussian = random_provider::gaussian(value, std_dev);

        let allele = GaussianMutator::clamp(gaussian, min, max) as f32;
        gene.with_allele(&allele)
    }
}
