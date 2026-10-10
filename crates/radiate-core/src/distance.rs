use crate::{
    Chromosome, ContiguousChromosome, Gene, PackedBitChromosome, Phenotype,
    bits::WORD_SIZE,
    chromosomes::{NumericAllele, gene::NumericGene},
    math::distance,
};

pub trait Distance<I>: Send + Sync
where
    I: ?Sized,
{
    fn calculate(&self, one: &I, two: &I) -> f32;
}

/// A concrete implementation of the [Distance] trait that calculates the Hamming distance
/// between two [crate::Genotype]s. The Hamming distance is the number of positions at which the
/// corresponding genes are different normalized by the total number of genes.
#[derive(Clone)]
pub struct HammingDistance;

impl<G, C> Distance<Phenotype<C>> for HammingDistance
where
    C: Chromosome<Gene = G>,
    G: Gene,
    G::Allele: PartialEq,
{
    #[inline]
    fn calculate(&self, geno_one: &Phenotype<C>, geno_two: &Phenotype<C>) -> f32 {
        let geno_one = geno_one.genotype();
        let geno_two = geno_two.genotype();

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

impl Distance<[f32]> for HammingDistance {
    fn calculate(&self, one: &[f32], two: &[f32]) -> f32 {
        distance::hamming(one, two)
    }
}

impl Distance<Vec<f32>> for HammingDistance {
    fn calculate(&self, one: &Vec<f32>, two: &Vec<f32>) -> f32 {
        distance::hamming(one, two)
    }
}

/// Bit-level Hamming distance for [`PackedBitChromosome`]s: the number of differing
/// bits normalized by the number of bits compared.
///
/// The generic [`HammingDistance`] compares whole 64-bit words on a packed chromosome,
/// so two words that differ in a single bit count the same as two that differ in all 64.
/// This one XORs the words and counts the set bits, so it's exact at the bit level
/// and still works 64 bits at a time.
///
/// Like [`HammingDistance`], chromosomes of different lengths are compared over their
/// common prefix. The unspecified tail bits are never counted. Comparing zero bits
/// gives a distance of `0.0`.
///
/// It also works on raw words (`[u64]` and `Vec<u64>`), such as the output of
/// [`PackedBitCodec`](crate::PackedBitCodec), which is what novelty search compares.
/// Raw words don't carry a bit count, so the distance is normalized by
/// `words * 64` instead of the number of bits, and the tail bits past the end of
/// the bit string are assumed to be zero. `PackedBitCodec` guarantees this, and
/// with zeroed tails the result differs from the exact bit-level fraction only by a
/// constant factor.
#[derive(Clone, Debug, PartialEq)]
pub struct PackedBitHammingDistance;

impl PackedBitHammingDistance {
    /// Returns `(differing bits, bits compared)` over the first
    /// `min(one.num_bits(), two.num_bits())` bits.
    #[inline]
    fn count(one: &PackedBitChromosome, two: &PackedBitChromosome) -> (usize, usize) {
        let num_bits = one.num_bits().min(two.num_bits());
        let (one, two) = (one.as_slice(), two.as_slice());

        let full_words = num_bits / WORD_SIZE;
        let mut differing = one[..full_words]
            .iter()
            .zip(&two[..full_words])
            .map(|(a, b)| (a.get() ^ b.get()).count_ones() as usize)
            .sum::<usize>();

        let tail_bits = num_bits % WORD_SIZE;
        if tail_bits != 0 {
            let mask = (1u64 << tail_bits) - 1;
            let diff = one[full_words].get() ^ two[full_words].get();
            differing += (diff & mask).count_ones() as usize;
        }

        (differing, num_bits)
    }
}

impl Distance<Phenotype<PackedBitChromosome>> for PackedBitHammingDistance {
    #[inline]
    fn calculate(
        &self,
        geno_one: &Phenotype<PackedBitChromosome>,
        geno_two: &Phenotype<PackedBitChromosome>,
    ) -> f32 {
        let (mut differing, mut total_bits) = (0, 0);
        for (one, two) in geno_one.genotype().iter().zip(geno_two.genotype().iter()) {
            let (diff, bits) = Self::count(one, two);
            differing += diff;
            total_bits += bits;
        }

        if total_bits == 0 {
            return 0.0;
        }

        differing as f32 / total_bits as f32
    }
}

impl Distance<[u64]> for PackedBitHammingDistance {
    #[inline]
    fn calculate(&self, one: &[u64], two: &[u64]) -> f32 {
        distance::packed_hamming(one, two)
    }
}

impl Distance<Vec<u64>> for PackedBitHammingDistance {
    #[inline]
    fn calculate(&self, one: &Vec<u64>, two: &Vec<u64>) -> f32 {
        <Self as Distance<[u64]>>::calculate(self, one, two)
    }
}

/// Implementation of the [Distance] trait that calculates the Euclidean distance
/// between two [crate::Genotype]s. The Euclidean distance is the square root of the sum of the
/// squared differences between the corresponding genes' alleles, normalized by the number of genes.
#[derive(Clone)]
pub struct EuclideanDistance;

impl<G, C> Distance<Phenotype<C>> for EuclideanDistance
where
    C: Chromosome<Gene = G>,
    G: NumericGene,
    G::Allele: NumericAllele,
{
    #[inline]
    fn calculate(&self, geno_one: &Phenotype<C>, geno_two: &Phenotype<C>) -> f32 {
        let geno_one = geno_one.genotype();
        let geno_two = geno_two.genotype();

        let mut distance = 0.0;
        let mut total_genes = 0.0;
        for (chrom_one, chrom_two) in geno_one.iter().zip(geno_two.iter()) {
            for (gene_one, gene_two) in chrom_one.iter().zip(chrom_two.iter()) {
                let one_as_f64 = gene_one.allele().extract::<f64>();
                let two_as_f64 = gene_two.allele().extract::<f64>();

                if let Some((one, two)) = one_as_f64.zip(two_as_f64) {
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

        (distance / total_genes).sqrt() as f32
    }
}

impl Distance<[f32]> for EuclideanDistance {
    fn calculate(&self, one: &[f32], two: &[f32]) -> f32 {
        distance::euclidean(one, two)
    }
}

impl Distance<Vec<f32>> for EuclideanDistance {
    fn calculate(&self, one: &Vec<f32>, two: &Vec<f32>) -> f32 {
        distance::euclidean(one, two)
    }
}

#[derive(Clone)]
pub struct CosineDistance;

impl<G, C> Distance<Phenotype<C>> for CosineDistance
where
    C: Chromosome<Gene = G>,
    G: NumericGene,
    G::Allele: NumericAllele,
{
    #[inline]
    fn calculate(&self, geno_one: &Phenotype<C>, geno_two: &Phenotype<C>) -> f32 {
        let geno_one = geno_one.genotype();
        let geno_two = geno_two.genotype();

        let mut dot_product = 0.0;
        let mut norm_one = 0.0;
        let mut norm_two = 0.0;

        for (chrom_one, chrom_two) in geno_one.iter().zip(geno_two.iter()) {
            for (gene_one, gene_two) in chrom_one.iter().zip(chrom_two.iter()) {
                let one_as_f64 = gene_one.allele().extract::<f64>();
                let two_as_f64 = gene_two.allele().extract::<f64>();

                if let Some((one, two)) = one_as_f64.zip(two_as_f64) {
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

        1.0 - (dot_product / (norm_one.sqrt() * norm_two.sqrt())) as f32
    }
}

impl Distance<[f32]> for CosineDistance {
    fn calculate(&self, one: &[f32], two: &[f32]) -> f32 {
        distance::cosine(one, two)
    }
}

impl Distance<Vec<f32>> for CosineDistance {
    fn calculate(&self, one: &Vec<f32>, two: &Vec<f32>) -> f32 {
        distance::cosine(one, two)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hamming_distance() {
        let distance = HammingDistance;
        let vec_one = vec![1.0, 2.0, 3.0];
        let vec_two = vec![1.0, 2.0, 4.0];

        assert_eq!(distance.calculate(vec_one.as_slice(), &vec_two), 1.0 / 3.0);
    }

    #[test]
    fn test_euclidean_distance() {
        let distance = EuclideanDistance;
        let vec_one = vec![1.0, 2.0, 3.0];
        let vec_two = vec![1.0, 2.0, 4.0];

        assert_eq!(distance.calculate(vec_one.as_slice(), &vec_two), 1.0);
    }

    #[test]
    fn test_cosine_distance() {
        let distance = CosineDistance;
        let vec_one = vec![1.0, 2.0, 3.0];
        let vec_two = vec![1.0, 2.0, 4.0];

        assert_eq!(
            distance.calculate(vec_one.as_slice(), vec_two.as_slice()),
            0.008539915
        );
    }
}
