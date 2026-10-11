use super::Valid;
use crate::Gene;
use std::ops::Range;

/// The [Chromosome] is part of the genetic makeup of an individual.
/// It is a collection of [Gene] instances, it is essentially a
/// light wrapper around a Vec of [Gene]s. The [Chromosome] struct, however, has some additional
/// functionality and terminology that aligns with the biological concept of a chromosome
///
/// In traditional biological terms, a [Chromosome] is a long DNA molecule with part or all of the
/// genetic material of an organism. The [Chromosome] is the 'genetic' part of the individual that is
/// being evolved by the genetic algorithm.
///
/// We can think of a [Chromosome] as a Vec of structs which implement the [Gene] trait. For example,
/// if we have a [Chromosome] with 3 [Gene]s, it is represented as follows:
/// ```text
/// Chromosome: [Gene, Gene, Gene]
/// ```
pub trait Chromosome: Valid {
    type Gene: Gene;

    fn iter(&self) -> impl Iterator<Item = &Self::Gene>;
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Self::Gene>;

    fn get(&self, index: usize) -> Option<&Self::Gene>;
    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Gene>;
    fn set(&mut self, index: usize, gene: Self::Gene);

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait ContiguousChromosome: Chromosome {
    fn as_slice(&self) -> &[Self::Gene];
    fn as_mut_slice(&mut self) -> &mut [Self::Gene];

    fn slice(&self, range: Range<usize>) -> &[Self::Gene] {
        &self.as_slice()[range]
    }
    fn slice_mut(&mut self, range: Range<usize>) -> &mut [Self::Gene] {
        &mut self.as_mut_slice()[range]
    }

    fn swap(&mut self, i: usize, j: usize) {
        self.as_mut_slice().swap(i, j);
    }
}
