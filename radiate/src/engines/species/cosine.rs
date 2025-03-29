use crate::{Chromosome, FloatChromosome, Gene, Genotype};

use super::Distance;

pub struct CosineDistance {
    threshold: f32,
}

impl CosineDistance {
    pub fn new(threshold: f32) -> Self {
        Self { threshold }
    }
}

impl Distance<FloatChromosome> for CosineDistance {
    fn threshold(&self) -> f32 {
        self.threshold
    }

    fn distance(&self, a: &Genotype<FloatChromosome>, b: &Genotype<FloatChromosome>) -> f32 {
        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for (a, b) in a.iter().zip(b.iter()) {
            for (gene_one, gene_two) in a.iter().zip(b.iter()) {
                dot_product += gene_one.allele() * gene_two.allele();
                norm_a += gene_one.allele() * gene_one.allele();
                norm_b += gene_two.allele() * gene_two.allele();
            }
        }

        let norm_a = norm_a.sqrt();
        let norm_b = norm_b.sqrt();

        let cosine = dot_product / (norm_a * norm_b);

        1.0 - cosine
    }
}
