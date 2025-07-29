use crate::{ArithmeticGene, Chromosome, Gene, Genotype, fitness::Novelty};

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

impl<T> Novelty<Vec<T>> for HammingDistance
where
    T: PartialEq + Send + Sync + Clone,
{
    type Descriptor = Vec<T>;

    fn description(&self, phenotype: &Vec<T>) -> Self::Descriptor {
        phenotype.clone()
    }

    fn distance(&self, a: &Self::Descriptor, b: &Self::Descriptor) -> f32 {
        let mut distance = 0.0;
        let mut total_genes = 0.0;

        for (gene_one, gene_two) in a.iter().zip(b.iter()) {
            total_genes += 1.0;
            if gene_one != gene_two {
                distance += 1.0;
            }
        }

        distance / total_genes
    }
}

/// Implementation of the [Diversity] trait that calculates the Euclidean distance
/// between two [Genotype]s. The Euclidean distance is the square root of the sum of the
/// squared differences between the corresponding genes' alleles, normalized by the number of genes.
#[derive(Clone)]
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

impl Novelty<Vec<f32>> for EuclideanDistance {
    type Descriptor = Vec<f32>;

    fn description(&self, phenotype: &Vec<f32>) -> Self::Descriptor {
        phenotype.clone()
    }

    fn distance(&self, a: &Self::Descriptor, b: &Self::Descriptor) -> f32 {
        if a.len() != b.len() {
            return f32::INFINITY;
        }

        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum::<f32>()
            .sqrt()
    }
}

pub struct CosineDistance;

impl<G: ArithmeticGene, C: Chromosome<Gene = G>> Diversity<C> for CosineDistance
where
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
    type Descriptor = Vec<f32>;

    fn description(&self, phenotype: &Vec<f32>) -> Self::Descriptor {
        phenotype.clone()
    }

    fn distance(&self, a: &Self::Descriptor, b: &Self::Descriptor) -> f32 {
        if a.len() != b.len() {
            return f32::INFINITY;
        }

        let dot_product = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
        let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 1.0;
        }

        1.0 - (dot_product / (norm_a * norm_b + 1e-8))
    }
}
