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

    fn zip<'a>(&'a mut self, other: &'a mut Self) -> ZippedChromosome<'a, Self>
    where
        Self: Sized,
    {
        ZippedChromosome {
            one: self,
            two: other,
        }
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

pub struct ZippedChromosome<'a, C: Chromosome> {
    one: &'a mut C,
    two: &'a mut C,
}

impl<'a, C: Chromosome> ZippedChromosome<'a, C> {
    pub fn new(one: &'a mut C, two: &'a mut C) -> Self {
        ZippedChromosome { one, two }
    }

    pub fn iter(&mut self) -> impl Iterator<Item = (&mut C::Gene, &mut C::Gene)> {
        self.one.iter_mut().zip(self.two.iter_mut())
    }

    pub fn swap(&mut self, i: usize)
    where
        C: ContiguousChromosome,
    {
        let one_gene = &mut self.one.as_mut_slice()[i];
        let two_gene = &mut self.two.as_mut_slice()[i];
        std::mem::swap(one_gene, two_gene);
    }

    pub fn for_each<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut C::Gene, &mut C::Gene),
    {
        for (gene_one, gene_two) in self.one.iter_mut().zip(self.two.iter_mut()) {
            f(gene_one, gene_two);
        }
    }
}
