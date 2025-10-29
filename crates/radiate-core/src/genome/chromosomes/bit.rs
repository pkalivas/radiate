use crate::{Chromosome, Gene, Valid, random_provider};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

/// A gene that represents a single bit. The `allele` is a `bool` that is randomly assigned.
/// The `allele` is either `true` or `false`. This is the simplest form of a gene and
/// in traditional genetic algorithms is the gene that is used to represent the individuals.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// // Craete a new BitGene from the allele `true`.
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
///
#[derive(Clone, PartialEq, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    /// The length is the number of genes in the chromosome.
    pub fn new(length: usize) -> Self {
        let genes = (0..length).map(|_| BitGene::new()).collect();
        BitChromosome { genes }
    }
}

impl Chromosome for BitChromosome {
    type Gene = BitGene;

    fn genes(&self) -> &[Self::Gene] {
        &self.genes
    }

    fn genes_mut(&mut self) -> &mut [Self::Gene] {
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
