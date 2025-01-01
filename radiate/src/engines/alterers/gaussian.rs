use crate::{random_provider, Chromosome, FloatGene, Gene, NumericGene};

use super::{AlterAction, CrossoverAction, EngineAlterer, EngineCompoment, MutateAction};

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

impl EngineCompoment for GaussianMutator {
    fn name(&self) -> &'static str {
        "GaussianMutator"
    }
}

impl<C: Chromosome<Gene = FloatGene>> EngineAlterer<C> for GaussianMutator {
    fn rate(&self) -> f32 {
        self.rate
    }

    fn to_alter(self) -> AlterAction<C> {
        AlterAction::Mutate(Box::new(self))
    }
}

impl<C: Chromosome<Gene = FloatGene>> MutateAction<C> for GaussianMutator {
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

// impl<C: Chromosome<Gene = FloatGene>> Alter<C> for GaussianMutator {
//     fn name(&self) -> &'static str {
//         "GaussianMutator"
//     }

//     fn rate(&self) -> f32 {
//         self.rate
//     }

//     fn alter_type(&self) -> AlterType {
//         AlterType::Mutator
//     }

//     #[inline]
//     fn mutate_gene(&self, gene: &C::Gene) -> C::Gene {
//         let min = *gene.min() as f64;
//         let max = *gene.max() as f64;

//         let std_dev = (max - min) * 0.25;
//         let value = *gene.allele() as f64;

//         let gaussian = random_provider::gaussian(value, std_dev);

//         let allele = GaussianMutator::clamp(gaussian, min, max) as f32;
//         gene.with_allele(&allele)
//     }
// }
