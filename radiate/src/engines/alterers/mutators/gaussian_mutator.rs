use crate::{random_provider, Chromosome, FloatGene, Gene, Mutate, NumericGene};

pub struct GaussianMutator {
    rate: f32,
}

impl GaussianMutator {
    pub fn new(rate: f32) -> Self {
        GaussianMutator { rate }
    }

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

impl<C: Chromosome<GeneType = FloatGene>> Mutate<C> for GaussianMutator
// where
// C::GeneType: NumericGene,
{
    fn mutate_rate(&self) -> f32 {
        self.rate
    }

    fn name(&self) -> &'static str {
        "GaussianMutator"
    }

    #[inline]
    fn mutate_gene(&self, gene: &C::GeneType) -> C::GeneType {
        let min = *gene.min() as f64;
        let max = *gene.max() as f64;

        let std_dev = (max - min) * 0.25;
        let value = *gene.allele() as f64;

        let gaussian = random_provider::gauss(value, std_dev);

        let allele = GaussianMutator::clamp(gaussian, min, max) as f32;
        gene.from_allele(&allele)
    }
}
