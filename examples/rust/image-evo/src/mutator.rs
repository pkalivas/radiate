use crate::chromosome::ImageChromosome;
use radiate::{Chromosome, Gene, Mutate, random_provider};

/// This is a simple mutator for the [ImageChromosome] that mutates the vertices of the polygons.
/// This is the same logic used by the [JitterMutator](radiate::JitterMutator) but adapted/applied to the [ImageGene].
#[derive(Clone, Debug)]
pub struct ImageMutator {
    rate: f32,
    magnitude: f32,
}

impl ImageMutator {
    pub fn new(rate: f32, magnitude: f32) -> Self {
        Self { rate, magnitude }
    }
}

impl Mutate<ImageChromosome> for ImageMutator {
    fn mutate_chromosome(
        &mut self,
        chromosome: &mut ImageChromosome,
        _: &mut radiate::prelude::AlterContext,
    ) -> radiate::prelude::AlterResult {
        let mut count = 0;
        for gene in chromosome.iter_mut() {
            for i in 0..gene.allele().len() {
                if random_provider::random::<f32>() < self.rate {
                    let change = (random_provider::random::<f32>() * 2.0 - 1.0) * self.magnitude;
                    gene.allele_mut()[i] = (gene.allele()[i] + change).clamp(0.0, 1.0);
                    count += 1;
                }
            }
        }

        count.into()
    }
}
