use crate::{ArithmeticGene, Chromosome, Gene, Genotype};

/// Trait for measuring diversity between two [Genotype]s.
/// Within radiate this is mostly used for speciation and determining how genetically
/// similar two individuals are. Through this, the engine can determine
/// whether two individuals belong to the same species or not.
pub trait Diversity<C: Chromosome>: Send + Sync {
    fn measure(&self, geno_one: &Genotype<C>, geno_two: &Genotype<C>) -> f32;
}

/// A concrete implementation of the [Diversity] trait that calculates the Hamming distance
/// between two [Genotype]s. The Hamming distance is the number of positions at which the
/// corresponding genes are different normalized by the total number of genes.
pub struct HammingDistance;

impl<G: Gene, C: Chromosome<Gene = G>> Diversity<C> for HammingDistance
where
    G::Allele: PartialEq,
{
    fn measure(&self, geno_one: &Genotype<C>, geno_two: &Genotype<C>) -> f32 {
        let mut distance = 0.0;
        let mut total_genes = 0.0;
        for (chrom_one, chrom_two) in geno_one.iter().zip(geno_two.iter()) {
            for (gene_one, gene_two) in chrom_one.iter().zip(chrom_two.iter()) {
                total_genes += 1.0;
                if gene_one.allele() != gene_two.allele() {
                    distance += 1.0;
                }
            }
        }

        distance / total_genes
    }
}

/// Implementation of the [Diversity] trait that calculates the Euclidean distance
/// between two [Genotype]s. The Euclidean distance is the square root of the sum of the
/// squared differences between the corresponding genes' alleles, normalized by the number of genes.
pub struct EuclideanDistance;

impl<G: ArithmeticGene, C: Chromosome<Gene = G>> Diversity<C> for EuclideanDistance
where
    G::Allele: Into<f32> + Copy,
{
    fn measure(&self, geno_one: &Genotype<C>, geno_two: &Genotype<C>) -> f32 {
        let mut distance = 0.0;
        let mut total_genes = 0.0;
        for (chrom_one, chrom_two) in geno_one.iter().zip(geno_two.iter()) {
            for (gene_one, gene_two) in chrom_one.iter().zip(chrom_two.iter()) {
                let one_as_f32: f32 = (*gene_one.allele()).into();
                let two_as_f32: f32 = (*gene_two.allele()).into();

                if one_as_f32.is_nan() || two_as_f32.is_nan() {
                    continue;
                }

                let diff = one_as_f32 - two_as_f32;
                distance += diff * diff;
                total_genes += 1.0;
            }
        }
        if total_genes == 0.0 {
            return 0.0;
        }

        (distance / total_genes).sqrt()
    }
}
