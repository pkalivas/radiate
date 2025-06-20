use crate::chromosome::{ImageChromosome, ImageGene};
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
    // fn rate(&self) -> f32 {
    //     self.rate
    // }

    // fn mutate_chromosome(&self, chromosome: &mut ImageChromosome, rate: f32) -> AlterResult {
    //     let mut alter_count = 0;
    //     for gene in chromosome.genes_mut() {
    //         for i in 0..gene.allele().len() {
    //             if random_provider::random::<f32>() < self.rate {
    //                 let change = (random_provider::random::<f32>() * 2.0 - 1.0) * self.magnitude;
    //                 gene.polygon_mut()[i] = (gene.allele()[i] + change).clamp(0.0, 1.0);
    //                 alter_count += 1;
    //             }
    //         }
    //     }

    //     alter_count.into()
    // }

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
