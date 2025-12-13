use crate::{
    Chromosome, Gene, Genotype,
    chromosomes::{NumericAllele, gene::NumericGene},
    fitness::Novelty,
    math::distance,
};
use std::sync::Arc;

pub trait Distance<T>: Send + Sync {
    fn distance(&self, one: &T, two: &T) -> f32;
}

pub struct DistanceDiversityAdapter<C: Chromosome> {
    diversity: Arc<dyn Diversity<C>>,
}

impl<C: Chromosome> DistanceDiversityAdapter<C> {
    pub fn new(diversity: Arc<dyn Diversity<C>>) -> Self {
        Self { diversity }
    }
}

impl<C: Chromosome> Distance<Genotype<C>> for DistanceDiversityAdapter<C> {
    fn distance(&self, one: &Genotype<C>, two: &Genotype<C>) -> f32 {
        self.diversity.measure(one, two)
    }
}

/// Trait for measuring diversity between two [Genotype]s.
/// Within radiate this is mostly used for speciation and determining how genetically
/// similar two individuals are. Through this, the engine can determine
/// whether two individuals belong to the same [Species](super::genome::species::Species) or not.
pub trait Diversity<C: Chromosome>: Send + Sync {
    fn measure(&self, geno_one: &Genotype<C>, geno_two: &Genotype<C>) -> f32;
}

impl<C: Chromosome, F> Diversity<C> for F
where
    F: Fn(&Genotype<C>, &Genotype<C>) -> f32 + Send + Sync,
{
    fn measure(&self, geno_one: &Genotype<C>, geno_two: &Genotype<C>) -> f32 {
        (self)(geno_one, geno_two)
    }
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

impl<P: AsRef<[f32]>> Distance<P> for HammingDistance {
    fn distance(&self, one: &P, two: &P) -> f32 {
        let vec_one = one.as_ref();
        let vec_two = two.as_ref();

        distance::hamming(vec_one, vec_two)
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
    G: NumericGene,
    G::Allele: NumericAllele,
{
    fn measure(&self, geno_one: &Genotype<C>, geno_two: &Genotype<C>) -> f32 {
        let mut distance = 0.0;
        let mut total_genes = 0.0;
        for (chrom_one, chrom_two) in geno_one.iter().zip(geno_two.iter()) {
            for (gene_one, gene_two) in chrom_one.iter().zip(chrom_two.iter()) {
                let one_as_f32 = gene_one.allele_as_f32();
                let two_as_f32 = gene_two.allele_as_f32();

                if let Some((one, two)) = one_as_f32.zip(two_as_f32) {
                    if one.is_nan() || two.is_nan() {
                        continue;
                    }

                    let diff = one - two;
                    distance += diff * diff;
                    total_genes += 1.0;
                }
            }
        }

        if total_genes == 0.0 {
            return 0.0;
        }

        (distance / total_genes).sqrt()
    }
}

impl<P: AsRef<[f32]>> Distance<P> for EuclideanDistance {
    fn distance(&self, one: &P, two: &P) -> f32 {
        let vec_one = one.as_ref();
        let vec_two = two.as_ref();

        distance::euclidean(vec_one, vec_two)
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
    G: NumericGene,
    G::Allele: NumericAllele,
{
    fn measure(&self, geno_one: &Genotype<C>, geno_two: &Genotype<C>) -> f32 {
        let mut dot_product = 0.0;
        let mut norm_one = 0.0;
        let mut norm_two = 0.0;

        for (chrom_one, chrom_two) in geno_one.iter().zip(geno_two.iter()) {
            for (gene_one, gene_two) in chrom_one.iter().zip(chrom_two.iter()) {
                let one_as_f32 = gene_one.allele_as_f32();
                let two_as_f32 = gene_two.allele_as_f32();

                if let Some((one, two)) = one_as_f32.zip(two_as_f32) {
                    if one.is_nan() || two.is_nan() {
                        continue;
                    }

                    dot_product += one * two;
                    norm_one += one * one;
                    norm_two += two * two;
                }
            }
        }

        if norm_one == 0.0 || norm_two == 0.0 {
            return 1.0;
        }

        1.0 - (dot_product / (norm_one.sqrt() * norm_two.sqrt()))
    }
}

impl<P: AsRef<[f32]>> Distance<P> for CosineDistance {
    fn distance(&self, one: &P, two: &P) -> f32 {
        let vec_one = one.as_ref();
        let vec_two = two.as_ref();

        distance::cosine(vec_one, vec_two)
    }
}

impl Novelty<Vec<f32>> for CosineDistance {
    fn description(&self, phenotype: &Vec<f32>) -> Vec<f32> {
        phenotype.clone()
    }
}
