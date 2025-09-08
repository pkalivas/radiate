use crate::chromosome::{ImageChromosome, ImageGene};
use radiate::{Gene, Mutate, random_provider};

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
    fn mutate_gene(&self, gene: &ImageGene) -> ImageGene {
        let mut new_polygon = gene.allele().clone();

        for i in 0..new_polygon.len() {
            if random_provider::random::<f32>() < self.rate {
                let change = (random_provider::random::<f32>() * 2.0 - 1.0) * self.magnitude;
                new_polygon[i] = (new_polygon[i] + change).clamp(0.0, 1.0);
            }
        }

        ImageGene::from(new_polygon)
    }
}
