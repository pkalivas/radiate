use num_traits::{FromPrimitive, ToPrimitive};
use radiate_core::{BoundedGene, Chromosome, Gene, Mutate, random_provider};

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
        if !(0.0..=1.0).contains(&rate) {
            panic!("Rate must be between 0 and 1");
        }

        GaussianMutator { rate }
    }
}

impl<C, G> Mutate<C> for GaussianMutator
where
    C: Chromosome<Gene = G>,
    G: BoundedGene,
    <G as Gene>::Allele: ToPrimitive + FromPrimitive + Copy,
{
    fn rate(&self) -> f32 {
        self.rate
    }

    #[inline]
    fn mutate_gene(&self, gene: &C::Gene) -> C::Gene {
        let min = gene.min().to_f64().unwrap_or(f64::NEG_INFINITY);
        let max = gene.max().to_f64().unwrap_or(f64::INFINITY);

        // default sigma = 25% of range, with a sane fallback
        let mut std_dev = (max - min) * 0.25f64;
        if !std_dev.is_finite() || std_dev == 0.0 {
            std_dev = 0.1;
        }

        let value = gene.allele().to_f64().unwrap_or(0.0);
        let gaussian = random_provider::gaussian(value, std_dev);
        let clamped = gaussian.clamp(min, max);

        let new_allele = <G as Gene>::Allele::from_f64(clamped)
            .unwrap_or_else(|| <G as Gene>::Allele::from_f64(value).unwrap());

        gene.with_allele(&new_allele)
    }
}

// impl<C: Chromosome<Gene = FloatGene>> Mutate<C> for GaussianMutator {
//     fn rate(&self) -> f32 {
//         self.rate
//     }

//     #[inline]
//     fn mutate_gene(&self, gene: &C::Gene) -> C::Gene {
//         let min = *gene.min() as f64;
//         let max = *gene.max() as f64;

//         let std_dev = (max - min) * 0.25;
//         let value = *gene.allele() as f64;

//         let gaussian = random_provider::gaussian(value, std_dev);
//         let allele = gaussian.clamp(min, max) as f32;

//         gene.with_allele(&allele)
//     }
// }
