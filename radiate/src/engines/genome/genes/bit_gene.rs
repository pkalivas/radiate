use crate::RandomProvider;

use super::gene::{Gene, Valid};

/// A gene that represents a single bit. The `allele` is a `bool` that is randomly assigned.
/// The `allele` is either `true` or `false`. This is the simplest form of a gene and 
/// in traditional genetic algorithms is the the gene that is used to represent the individuals.
/// 
/// # Example
/// ``` rust
/// use radiate::*;
/// 
/// // Create a new BitGene with a random allele.
/// let gene = BitGene::new();
/// 
/// // Get the allele of the BitGene.
/// let allele = gene.allele();
/// 
/// // Create a new BitGene from the allele.
/// let gene = gene.from_allele(allele);
/// ```
pub struct BitGene {
    allele: bool,
}

impl BitGene {
    pub fn new() -> Self {
        BitGene {
            allele: RandomProvider::gen_range(0..2) == 1,
        }
    }
}

impl Gene<BitGene, bool> for BitGene {
    fn allele(&self) -> &bool {
        &self.allele
    }

    fn new_instance(&self) -> BitGene {
        BitGene::new()
    }

    fn from_allele(&self, allele: &bool) -> BitGene {
        BitGene {
            allele: allele.clone(),
        }
    }
}

impl Valid for BitGene {}

impl Clone for BitGene {
    fn clone(&self) -> Self {
        BitGene {
            allele: self.allele,
        }
    }
}

impl PartialEq for BitGene {
    fn eq(&self, other: &Self) -> bool {
        self.allele == other.allele
    }
}

impl std::fmt::Debug for BitGene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if self.allele { 1 } else { 0 })
    }
}

impl Into<BitGene> for bool {
    fn into(self) -> BitGene {
        BitGene { allele: self }
    }
}

impl Into<bool> for BitGene {
    fn into(self) -> bool {
        self.allele
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_new() {
        let gene = BitGene::new();
        assert!(gene.allele == true || gene.allele == false);
    }

    #[test]
    fn test_allele() {
        let gene = BitGene::new();
        assert!(gene.allele() == &gene.allele);
    }

    #[test]
    fn test_into() {
        let gene = BitGene::new();
        let copy = gene.clone();
        let allele: bool = gene.into();
        assert!(allele == copy.allele);
    }

    #[test]
    fn test_from() {
        let gene = BitGene::new();
        let copy = gene.clone();
        let allele: BitGene = copy.into();
        assert!(allele == gene);
    }
}
