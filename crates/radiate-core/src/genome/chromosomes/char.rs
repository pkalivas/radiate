use super::{
    Chromosome,
    gene::{Gene, Valid},
};
use crate::random_provider;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{char, sync::Arc};

/// This is the default character set used for the `CharGene` and `CharChromosome`.
/// It includes digits, lowercase and uppercase letters, and a selection of special characters.
/// The character set is designed to be broad enough for most applications while still being
/// manageable in size to ensure good performance in genetic algorithms.
///
/// If no character set is provided, this default will be used.
pub(crate) const ALPHABET: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"$%&/()=?`{[]}\\+~*#';.:,-_<>|@^' ";

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
#[derive(Clone, PartialEq)]
pub struct CharGene {
    allele: char,
    char_set: Arc<[char]>,
}

impl CharGene {
    /// Given a slice of possible alleles, create a new [CharGene] by
    /// randomly picking a char from the char_set
    pub fn new(char_set: Arc<[char]>) -> Self {
        let index = random_provider::range(0..char_set.len());
        CharGene {
            allele: char_set[index],
            char_set,
        }
    }

    /// Get this [CharGene]'s set of possible alleles
    pub fn char_set(&self) -> &[char] {
        &self.char_set
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

    fn allele_mut(&mut self) -> &mut char {
        &mut self.allele
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

/// Manual implementation of seralize and deserialzie for the [CharGene]
/// needed because of the [Arc] type in the char_set field.
#[cfg(feature = "serde")]
impl Serialize for CharGene {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("CharGene", 2)?;
        state.serialize_field("allele", &self.allele)?;
        state.serialize_field("char_set", &self.char_set.to_vec())?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for CharGene {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct CharGeneData {
            allele: char,
            char_set: Vec<char>,
        }

        let data = CharGeneData::deserialize(deserializer)?;
        Ok(CharGene {
            allele: data.allele,
            char_set: data.char_set.into(),
        })
    }
}

/// A [`Chromosome`] that contains [`CharGene`].
#[derive(Clone, PartialEq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CharChromosome {
    genes: Vec<CharGene>,
}

impl CharChromosome {
    pub fn new(genes: Vec<CharGene>) -> Self {
        CharChromosome { genes }
    }
}

impl Chromosome for CharChromosome {
    type Gene = CharGene;

    fn genes(&self) -> &[Self::Gene] {
        &self.genes
    }

    fn genes_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.genes
    }
}

impl Valid for CharChromosome {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}

impl From<CharGene> for CharChromosome {
    fn from(gene: CharGene) -> Self {
        CharChromosome { genes: vec![gene] }
    }
}

impl From<Vec<CharGene>> for CharChromosome {
    fn from(genes: Vec<CharGene>) -> Self {
        CharChromosome { genes }
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

impl FromIterator<CharGene> for CharChromosome {
    fn from_iter<I: IntoIterator<Item = CharGene>>(iter: I) -> Self {
        CharChromosome {
            genes: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for CharChromosome {
    type Item = CharGene;
    type IntoIter = std::vec::IntoIter<CharGene>;

    fn into_iter(self) -> Self::IntoIter {
        self.genes.into_iter()
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
