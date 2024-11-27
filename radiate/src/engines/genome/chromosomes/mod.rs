pub mod bit_chromosome;
pub mod char_chromosome;
pub mod float_chromosome;
pub mod int_chromosome;
pub mod permutation_chromosome;

pub use bit_chromosome::*;
pub use char_chromosome::*;
pub use float_chromosome::*;
pub use int_chromosome::*;
pub use permutation_chromosome::*;

use super::genes::gene::Gene;
use crate::Valid;

/// The `Chromosome` struct represents a collection of `Gene` instances. The `Chromosome` is part of the
/// genetic makeup of an individual. It is a collection of `Gene` instances, it is essentially a
/// light wrapper around a Vec of `Gene`s. The `Chromosome` struct, however, has some additional
/// functionality and terminology that aligns with the biological concept of a chromosome.
///
/// In traditional biological terms, a `Chromosome` is a long DNA molecule with part or all of the
/// genetic material of an organism. The `Chromosome` is the 'genetic' part of the individual that is
/// being evolved by the genetic algorithm.
///
/// We can think of a `Chromosome` as a Vec of structs which implement the `Gene` trait. For example,
/// if we have a `Chromosome` with 3 `Gene`s, it is represented as follows:
/// ```text
/// Chromosome: [Gene, Gene, Gene]
/// ```
///
pub trait Chromosome: Clone + PartialEq + Valid {
    type GeneType: Gene;

    fn from_genes(genes: Vec<Self::GeneType>) -> Self;
    fn set_gene(&mut self, index: usize, gene: Self::GeneType);
    fn get_genes(&self) -> &[Self::GeneType];
    fn get_genes_mut(&mut self) -> &mut [Self::GeneType];

    fn get_gene(&self, index: usize) -> &Self::GeneType {
        &self.get_genes()[index]
    }

    fn len(&self) -> usize {
        self.get_genes().len()
    }

    fn iter(&self) -> std::slice::Iter<Self::GeneType> {
        self.get_genes().iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<Self::GeneType> {
        self.get_genes_mut().iter_mut()
    }

    fn is_empty(&self) -> bool {
        self.get_genes().is_empty()
    }
}
