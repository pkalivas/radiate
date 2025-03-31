use super::DiversityMeasure;
use crate::{Chromosome, FloatChromosome, Gene, Phenotype};

pub struct CosineDistance;

impl DiversityMeasure<FloatChromosome> for CosineDistance {
    fn diversity(&self, a: &Phenotype<FloatChromosome>, b: &Phenotype<FloatChromosome>) -> f32 {
        let mut dot_product = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for (a, b) in a.genotype().iter().zip(b.genotype().iter()) {
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
