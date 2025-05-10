use super::{
    Chromosome,
    gene::{Gene, Valid},
};
use crate::random_provider;
use std::{char, sync::Arc};

pub const ALPHABET: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"$%&/()=?`{[]}\\+~*#';.:,-_<>|@^' ";

/// A gene that represents a single character. The `allele` is a `char`
/// that is randomly selected from the [`ALPHABET`] constant.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new CharGene with a random allele from the ALPHABET constant.
/// let gene = CharGene::default();
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
    pub char_set: Arc<[char]>,
}

impl CharGene {
    pub fn new(char_set: Arc<[char]>) -> Self {
        let index = random_provider::range(0..char_set.len());
        CharGene {
            allele: char_set[index],
            char_set,
        }
    }
}

/// Implement the [`Gene`] trait for the [`CharGene`]. This allows the [`CharGene`] to be used in
/// a [`Chromosome`] - specifically the [`CharChromosome`], thus allowing the [`CharGene`] to
/// be used in the `GeneticEngine`.
impl Gene for CharGene {
    type Allele = char;

    fn allele(&self) -> &char {
        &self.allele
    }

    fn new_instance(&self) -> CharGene {
        let index = random_provider::range(0..self.char_set.len());
        CharGene {
            allele: self.char_set[index],
            char_set: Arc::clone(&self.char_set),
        }
    }

    fn with_allele(&self, allele: &char) -> CharGene {
        CharGene {
            allele: *allele,
            char_set: Arc::clone(&self.char_set),
        }
    }
}

impl Valid for CharGene {
    fn is_valid(&self) -> bool {
        self.char_set.contains(&self.allele)
    }
}

impl Default for CharGene {
    fn default() -> Self {
        let char_set: Arc<[char]> = ALPHABET.chars().collect::<Vec<char>>().into();
        let allele = random_provider::range(0..char_set.len());
        CharGene {
            allele: char_set[allele],
            char_set,
        }
    }
}

impl From<CharGene> for char {
    fn from(gene: CharGene) -> char {
        gene.allele
    }
}

impl From<char> for CharGene {
    fn from(allele: char) -> Self {
        CharGene {
            allele,
            char_set: ALPHABET.chars().collect::<Vec<char>>().into(),
        }
    }
}

impl From<&str> for CharGene {
    fn from(str: &str) -> Self {
        let char_set: Arc<[char]> = str.chars().collect::<Vec<char>>().into();
        let allele = random_provider::range(0..char_set.len());
        CharGene {
            allele: char_set[allele],
            char_set,
        }
    }
}

impl From<String> for CharGene {
    fn from(string: String) -> Self {
        let char_set: Arc<[char]> = string.chars().collect::<Vec<char>>().into();
        let allele = random_provider::range(0..char_set.len());
        CharGene {
            allele: char_set[allele],
            char_set,
        }
    }
}

impl From<(char, Arc<[char]>)> for CharGene {
    fn from((allele, char_set): (char, Arc<[char]>)) -> Self {
        CharGene { allele, char_set }
    }
}

impl std::fmt::Debug for CharGene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.allele)
    }
}

/// A [`Chromosome`] that contains [`CharGene`].
#[derive(Clone, PartialEq, Default, Debug)]
pub struct CharChromosome {
    pub genes: Vec<CharGene>,
}

impl Chromosome for CharChromosome {
    type Gene = CharGene;
}

impl Valid for CharChromosome {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}

impl AsRef<[CharGene]> for CharChromosome {
    fn as_ref(&self) -> &[CharGene] {
        &self.genes
    }
}

impl AsMut<[CharGene]> for CharChromosome {
    fn as_mut(&mut self) -> &mut [CharGene] {
        &mut self.genes
    }
}

impl From<String> for CharChromosome {
    fn from(alleles: String) -> Self {
        let char_set: Arc<[char]> = alleles.chars().collect::<Vec<char>>().into();
        let genes = char_set
            .iter()
            .map(|&allele| CharGene::from((allele, Arc::clone(&char_set))))
            .collect();
        CharChromosome { genes }
    }
}

impl From<&str> for CharChromosome {
    fn from(alleles: &str) -> Self {
        let char_set: Arc<[char]> = alleles.chars().collect::<Vec<char>>().into();
        let genes = char_set
            .iter()
            .map(|&allele| CharGene::from((allele, Arc::clone(&char_set))))
            .collect();
        CharChromosome { genes }
    }
}

impl<T: Into<String>> From<(usize, Option<T>)> for CharChromosome {
    fn from((length, alleles): (usize, Option<T>)) -> Self {
        let char_set: Arc<[char]> = alleles
            .map(|chars| chars.into().chars().collect::<Vec<char>>())
            .unwrap_or_else(|| ALPHABET.chars().collect::<Vec<char>>())
            .into();
        let genes = (0..length)
            .map(|_| CharGene::new(Arc::clone(&char_set)))
            .collect();
        CharChromosome { genes }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let gene = CharGene::default();
        assert!(ALPHABET.contains(gene.allele));
    }

    #[test]
    fn test_into() {
        let gene = CharGene::default();
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

    #[test]
    fn test_is_valid() {
        let gene_one = CharGene::default();
        let gene_two = CharGene::from("hello");
        let gene_three = gene_two.with_allele(&'a');

        assert!(gene_one.is_valid());
        assert!(gene_two.is_valid());
        assert!(!gene_three.is_valid());
    }

    #[test]
    fn test_char_from_str() {
        let gene = CharGene::from("hello");
        assert_eq!(
            "hello".chars().collect::<Vec<char>>(),
            gene.char_set.as_ref()
        );
    }

    #[test]
    fn test_char_chromosome_from_str() {
        let gene = CharChromosome::from("hello");
        assert_eq!(
            "hello".chars().collect::<Vec<char>>(),
            gene.genes.iter().map(|g| g.allele).collect::<Vec<char>>()
        );
    }

    #[test]
    fn test_char_chromosome_from_string() {
        let chromosome = CharChromosome::from("hello");
        let hello: String = chromosome.genes.iter().map(|g| g.allele).collect();

        assert_eq!(hello, "hello");
    }
}
