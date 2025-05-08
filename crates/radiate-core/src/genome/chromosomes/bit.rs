use crate::{Chromosome, Gene, Valid, random_provider};
use std::fmt::Debug;

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
#[derive(Clone, PartialEq)]
pub struct BitGene {
    allele: bool,
}

impl BitGene {
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

    fn new_instance(&self) -> BitGene {
        BitGene::new()
    }

    fn with_allele(&self, allele: &bool) -> BitGene {
        BitGene { allele: *allele }
    }
}

/// Because a [`BitGene`] is either `true` or `false` it is always valid.
impl Valid for BitGene {}

impl Default for BitGene {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for BitGene {
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
///
#[derive(Clone, PartialEq, Default, Debug)]
pub struct BitChromosome {
    pub genes: Vec<BitGene>,
}

impl Chromosome for BitChromosome {
    type Gene = BitGene;
}

impl Valid for BitChromosome {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}

impl AsRef<[BitGene]> for BitChromosome {
    fn as_ref(&self) -> &[BitGene] {
        &self.genes
    }
}

impl AsMut<[BitGene]> for BitChromosome {
    fn as_mut(&mut self) -> &mut [BitGene] {
        &mut self.genes
    }
}

impl From<Vec<bool>> for BitChromosome {
    fn from(alleles: Vec<bool>) -> Self {
        let genes = alleles.into_iter().map(BitGene::from).collect();
        BitChromosome { genes }
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
}
