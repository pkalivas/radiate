use crate::random_provider;

use super::gene::{Gene, Valid};

const ALPHABET: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"$%&/()=?`{[]}\\+~*#';.:,-_<>|@^' ";

/// A gene that represents a single character. The `allele` is a `char`
/// that is randomly selected from the `ALPHABET` constant.
///
/// # Example
/// ``` rust
/// use radiate::*;
///
/// // Create a new CharGene with a random allele from the ALPHABET constant.
/// let gene = CharGene::new();
///
/// // Get the allele of the CharGene.
/// let allele = gene.allele();
///
/// // Create a new CharGene from the allele.
/// let gene = gene.with_allele(allele);
/// ```
///
#[derive(Clone, PartialEq)]
pub struct CharGene {
    pub allele: char,
}

impl CharGene {
    pub fn new() -> Self {
        let index = random_provider::random::<usize>() % ALPHABET.len();
        CharGene {
            allele: ALPHABET.chars().nth(index).unwrap(),
        }
    }
}

impl Gene for CharGene {
    type Allele = char;

    fn allele(&self) -> &char {
        &self.allele
    }

    fn new_instance(&self) -> CharGene {
        CharGene::new()
    }

    fn with_allele(&self, allele: &char) -> CharGene {
        CharGene { allele: *allele }
    }
}

impl Valid for CharGene {}

impl std::fmt::Debug for CharGene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.allele)
    }
}

impl From<CharGene> for char {
    fn from(gene: CharGene) -> char {
        gene.allele
    }
}

impl From<char> for CharGene {
    fn from(allele: char) -> Self {
        CharGene { allele }
    }
}

impl From<&char> for CharGene {
    fn from(allele: &char) -> Self {
        CharGene { allele: *allele }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let gene = CharGene::new();
        assert!(ALPHABET.contains(gene.allele));
    }

    #[test]
    fn test_into() {
        let gene = CharGene::new();
        let copy = gene.clone();
        let allele: char = gene.into();
        assert_eq!(allele, copy.allele);
    }

    #[test]
    fn test_from() {
        let allele = 'a';
        let gene = CharGene::from(allele);
        assert_eq!(gene.allele, allele);
    }

    #[test]
    fn test_from_allele() {
        let gene_one: CharGene = 'a'.into();
        let gene_two: CharGene = 'b'.into();
        let new_gene = gene_one.with_allele(&gene_two.allele);

        assert_eq!(gene_two.allele, new_gene.allele);
    }
}
