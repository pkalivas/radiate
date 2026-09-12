use crate::{Chromosome, Gene, Valid, chromosomes::ContiguousChromosome, random_provider};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

const WORD_SIZE: u64 = 64;

/// A gene that represents a single bit. The `allele` is a `bool` that is randomly assigned.
/// The `allele` is either `true` or `false`. This is the simplest form of a gene and
/// in traditional genetic algorithms is the gene that is used to represent the individuals.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new BitGene from the allele `true`.
/// let gene: BitGene = true.into();
///
/// // Create a new BitGene with a random allele.
/// let gene = BitGene::new();
///
/// // Get the allele (bool) of the BitGene.
/// let allele = gene.allele();
///
/// // Create a new BitGene from the allele.
/// let gene = gene.with_allele(allele);
/// ```
#[derive(Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct BitGene {
    allele: bool,
}

impl BitGene {
    /// Create a new [`BitGene`] with a random allele.
    pub fn new() -> Self {
        BitGene {
            allele: random_provider::bool(0.5),
        }
    }

    pub fn flip(&mut self) {
        self.allele = !self.allele;
    }
}

/// Implement the [`Gene`] trait for the [`BitGene`].
/// This allows the [`BitGene`] to be used in a [`Chromosome`] - specifically the
/// [`BitChromosome`], thus allowing the [`BitGene`] to be used in the `GeneticEngine`.
impl Gene for BitGene {
    type Allele = bool;

    fn allele(&self) -> &bool {
        &self.allele
    }

    fn allele_mut(&mut self) -> &mut bool {
        &mut self.allele
    }

    fn new_instance(&self) -> BitGene {
        BitGene::new()
    }

    fn with_allele(&self, allele: &bool) -> BitGene {
        BitGene { allele: *allele }
    }

    fn set_allele(&mut self, allele: bool) {
        self.allele = allele;
    }
}

/// Because a [`BitGene`] is either `true` or `false` it is always valid.
impl Valid for BitGene {}

impl Display for BitGene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if self.allele { 1 } else { 0 })
    }
}

impl From<BitGene> for bool {
    fn from(gene: BitGene) -> bool {
        gene.allele
    }
}

impl From<bool> for BitGene {
    fn from(allele: bool) -> BitGene {
        BitGene { allele }
    }
}

/// A [`Chromosome`] that contains [`BitGene`].
/// A [`BitChromosome`] is a collection of [`BitGene`] that represent the genetic
/// material of an individual in the population.
#[derive(Clone, PartialEq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BitChromosome {
    genes: Vec<BitGene>,
}

impl BitChromosome {
    /// Create a new [`BitChromosome`] with the given length.
    pub fn new(length: usize) -> Self {
        BitChromosome {
            genes: (0..length).map(|_| BitGene::new()).collect(),
        }
    }

    pub fn pack(&self) -> Vec<u64> {
        let num_words = (self.genes.len() + (WORD_SIZE - 1) as usize) / WORD_SIZE as usize;
        let mut packed = Vec::with_capacity(num_words);
        let mut current = 0_u64;
        let mut count = 0;

        for gene in &self.genes {
            current = (current << 1) | (gene.allele as u64);
            count += 1;
            if count == WORD_SIZE {
                packed.push(current);
                current = 0;
                count = 0;
            }
        }

        if count > 0 {
            current <<= WORD_SIZE - count;
            packed.push(current);
        }

        packed
    }

    pub fn unpack(chunks: &[u64], length: usize) -> Self {
        let mut genes = Vec::with_capacity(length);

        for (i, &word) in chunks.iter().enumerate() {
            let remaining = (length - genes.len()) as u64;
            let bits_in_word = remaining.min(WORD_SIZE);

            for shift in (WORD_SIZE - bits_in_word..WORD_SIZE).rev() {
                genes.push(BitGene::from(((word >> shift) & 1) == 1));
            }

            debug_assert!(
                bits_in_word == WORD_SIZE || i == chunks.len() - 1,
                "partial chunk should only occur as the last chunk"
            );
        }

        BitChromosome { genes }
    }
}
impl Chromosome for BitChromosome {
    type Gene = BitGene;

    fn iter(&self) -> impl Iterator<Item = &Self::Gene> {
        self.genes.iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Self::Gene> {
        self.genes.iter_mut()
    }

    fn get(&self, index: usize) -> Option<&Self::Gene> {
        self.genes.get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Gene> {
        self.genes.get_mut(index)
    }

    fn set(&mut self, index: usize, gene: Self::Gene) {
        if let Some(slot) = self.genes.get_mut(index) {
            *slot = gene;
        }
    }

    fn len(&self) -> usize {
        self.genes.len()
    }
}

impl ContiguousChromosome for BitChromosome {
    fn as_slice(&self) -> &[Self::Gene] {
        &self.genes
    }

    fn as_mut_slice(&mut self) -> &mut [Self::Gene] {
        &mut self.genes
    }
}

/// Every `BitGene` is valid, so the `BitChromosome` is also valid.
impl Valid for BitChromosome {
    fn is_valid(&self) -> bool {
        true
    }
}

impl From<BitGene> for BitChromosome {
    fn from(gene: BitGene) -> Self {
        BitChromosome { genes: vec![gene] }
    }
}

impl From<Vec<BitGene>> for BitChromosome {
    fn from(genes: Vec<BitGene>) -> Self {
        BitChromosome { genes }
    }
}

impl From<Vec<bool>> for BitChromosome {
    fn from(alleles: Vec<bool>) -> Self {
        BitChromosome {
            genes: alleles.into_iter().map(BitGene::from).collect(),
        }
    }
}

impl FromIterator<BitGene> for BitChromosome {
    fn from_iter<I: IntoIterator<Item = BitGene>>(iter: I) -> Self {
        BitChromosome {
            genes: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for BitChromosome {
    type Item = BitGene;
    type IntoIter = std::vec::IntoIter<BitGene>;

    fn into_iter(self) -> Self::IntoIter {
        self.genes.into_iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_into() {
        let gene = BitGene::new();
        let copy = gene.clone();
        let allele: bool = gene.into();
        assert_eq!(allele, copy.allele);
    }

    #[test]
    fn test_from() {
        let gene = BitGene::new();
        let copy = gene.clone();
        let allele: BitGene = copy;
        assert_eq!(allele, gene);
    }

    #[test]
    fn test_from_allele() {
        let gene = BitGene::new();
        let copy = gene.clone();
        let allele = gene.allele();
        let new_gene = gene.with_allele(allele);
        assert_eq!(new_gene, copy);
    }

    #[test]
    fn test_pack_round_trip() {
        let chromosome = BitChromosome::from(vec![true, false, true, true, false]);
        let packed = chromosome.pack();
        let unpacked = BitChromosome::unpack(&packed, 5);

        assert_eq!(chromosome, unpacked);
    }

    #[test]
    fn test_pack_known_value() {
        let chromosome = BitChromosome::from(vec![true, false, true, true]);
        assert_eq!(chromosome.pack(), vec![0b1011u64 << 60]);
    }

    #[test]
    fn test_pack_empty() {
        let chromosome = BitChromosome::from(Vec::<bool>::new());
        assert_eq!(chromosome.pack(), Vec::<u64>::new());
    }

    #[test]
    fn test_unpack_length_shorter_than_64() {
        let chromosome = BitChromosome::from(vec![true, false, true]);
        let packed = chromosome.pack();
        let unpacked = BitChromosome::unpack(&packed, 3);
        assert_eq!(unpacked, chromosome);
    }

    #[test]
    fn test_chromosome_new_len() {
        let chromosome = BitChromosome::new(10);
        assert_eq!(chromosome.len(), 10);
    }

    #[test]
    fn test_chromosome_from_vec_bool() {
        let chromosome = BitChromosome::from(vec![true, false, true]);
        assert_eq!(chromosome.get(0).unwrap().allele(), &true);
        assert_eq!(chromosome.get(1).unwrap().allele(), &false);
        assert_eq!(chromosome.get(2).unwrap().allele(), &true);
    }

    #[test]
    fn test_chromosome_get_set() {
        let mut chromosome = BitChromosome::new(3);
        chromosome.set(1, BitGene::from(true));
        assert_eq!(chromosome.get(1).unwrap().allele(), &true);
    }

    #[test]
    fn test_pack_chunk_count() {
        assert_eq!(BitChromosome::new(64).pack().len(), 1);
        assert_eq!(BitChromosome::new(65).pack().len(), 2);
        assert_eq!(BitChromosome::new(128).pack().len(), 2);
        assert_eq!(BitChromosome::new(129).pack().len(), 3);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_bit_gene_serialization() {
        let gene = BitGene::new();
        let serialized = serde_json::to_string(&gene).expect("Failed to serialize BitGene");
        let deserialized: BitGene =
            serde_json::from_str(&serialized).expect("Failed to deserialize BitGene");

        let chromosome = BitChromosome::new(10);
        let serialized_chromosome =
            serde_json::to_string(&chromosome).expect("Failed to serialize BitChromosome");
        let deserialized_chromosome: BitChromosome = serde_json::from_str(&serialized_chromosome)
            .expect("Failed to deserialize BitChromosome");

        assert_eq!(gene, deserialized);
        assert_eq!(chromosome, deserialized_chromosome);
    }
}
