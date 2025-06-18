use crate::chromosome::ImageChromosome;
use radiate::{Gene, Mutate, random_provider};

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
    fn mutate_gene(
        &self,
        gene: &<ImageChromosome as radiate::Chromosome>::Gene,
    ) -> <ImageChromosome as radiate::Chromosome>::Gene {
        let mut new_polygon = gene.allele().clone();

        for i in 0..new_polygon.len() {
            if random_provider::random::<f32>() < self.rate {
                let change = (random_provider::random::<f32>() * 2.0 - 1.0) * self.magnitude;
                new_polygon[i] = (new_polygon[i] + change).clamp(0.0, 1.0);
            }
        }

        gene.with_allele(&new_polygon)
    }
}

// // Different mutation rates for different parameters
// let color_rate = self.rate * 0.9; // Less frequent color changes
// let position_rate = self.rate * 1.1; // More frequent position changes

// // Mutate colors (indices 0-3)
// for i in 0..4 {
//     if random_provider::random::<f32>() < color_rate {
//         let change = (random_provider::random::<f32>() * 2.0 - 1.0) * self.magnitude * 0.5;
//         new_polygon[i] = (new_polygon[i] + change).clamp(0.0, 1.0);
//     }
// }

// // Mutate positions (indices 4+)
// for i in 4..new_polygon.len() {
//     if random_provider::random::<f32>() < position_rate {
//         let change = (random_provider::random::<f32>() * 2.0 - 1.0) * self.magnitude;
//         new_polygon[i] = (new_polygon[i] + change).clamp(0.0, 1.0);
//     }
// }
