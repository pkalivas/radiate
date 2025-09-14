use crate::{ArithmeticGene, Chromosome, Gene, Genotype, fitness::Novelty};

/// Trait for measuring diversity between two [Genotype]s.
/// Within radiate this is mostly used for speciation and determining how genetically
/// similar two individuals are. Through this, the engine can determine
/// whether two individuals belong to the same [Species](super::genome::species::Species) or not.
pub trait Diversity<C: Chromosome>: Send + Sync {
    fn measure(&self, geno_one: &Genotype<C>, geno_two: &Genotype<C>) -> f32;
}

/// A concrete implementation of the [Diversity] trait that calculates the Hamming distance
/// between two [Genotype]s. The Hamming distance is the number of positions at which the
/// corresponding genes are different normalized by the total number of genes.
#[derive(Clone)]
pub struct HammingDistance;

impl<G, C> Diversity<C> for HammingDistance
where
    C: Chromosome<Gene = G>,
    G: Gene,
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

impl Novelty<Vec<f32>> for HammingDistance {
    fn description(&self, phenotype: &Vec<f32>) -> Vec<f32> {
        phenotype.clone()
    }
}

/// Implementation of the [Diversity] trait that calculates the Euclidean distance
/// between two [Genotype]s. The Euclidean distance is the square root of the sum of the
/// squared differences between the corresponding genes' alleles, normalized by the number of genes.
#[derive(Clone)]
pub struct EuclideanDistance;

impl<G, C> Diversity<C> for EuclideanDistance
where
    C: Chromosome<Gene = G>,
    G: ArithmeticGene,
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

impl Novelty<Vec<f32>> for EuclideanDistance {
    fn description(&self, phenotype: &Vec<f32>) -> Vec<f32> {
        phenotype.clone()
    }
}

#[derive(Clone)]
pub struct CosineDistance;

impl<G, C> Diversity<C> for CosineDistance
where
    C: Chromosome<Gene = G>,
    G: ArithmeticGene,
    G::Allele: Into<f32> + Copy,
{
    fn measure(&self, geno_one: &Genotype<C>, geno_two: &Genotype<C>) -> f32 {
        let mut dot_product = 0.0;
        let mut norm_one = 0.0;
        let mut norm_two = 0.0;

        for (chrom_one, chrom_two) in geno_one.iter().zip(geno_two.iter()) {
            for (gene_one, gene_two) in chrom_one.iter().zip(chrom_two.iter()) {
                let one_as_f32: f32 = (*gene_one.allele()).into();
                let two_as_f32: f32 = (*gene_two.allele()).into();

                if one_as_f32.is_nan() || two_as_f32.is_nan() {
                    continue;
                }

                dot_product += one_as_f32 * two_as_f32;
                norm_one += one_as_f32 * one_as_f32;
                norm_two += two_as_f32 * two_as_f32;
            }
        }

        if norm_one == 0.0 || norm_two == 0.0 {
            return 1.0;
        }

        1.0 - (dot_product / (norm_one.sqrt() * norm_two.sqrt()))
    }
}

impl Novelty<Vec<f32>> for CosineDistance {
    fn description(&self, phenotype: &Vec<f32>) -> Vec<f32> {
        phenotype.clone()
    }
}
