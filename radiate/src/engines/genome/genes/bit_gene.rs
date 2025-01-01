use super::gene::{Gene, Valid};
use crate::random_provider;

/// A gene that represents a single bit. The `allele` is a `bool` that is randomly assigned.
/// The `allele` is either `true` or `false`. This is the simplest form of a gene and
/// in traditional genetic algorithms is the gene that is used to represent the individuals.
///
/// # Example
/// ``` rust
/// use radiate::*;
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
            allele: random_provider::gen_range(0..2) == 1,
        }
    }
}

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

/// Because a `BitGene` is either `true` or `false` it is always valid.
impl Valid for BitGene {}

impl Default for BitGene {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for BitGene {
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

impl From<&bool> for BitGene {
    fn from(allele: &bool) -> BitGene {
        BitGene { allele: *allele }
    }
}

impl From<&u8> for BitGene {
    fn from(allele: &u8) -> BitGene {
        BitGene {
            allele: *allele == 1,
        }
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
