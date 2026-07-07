use radiate_utils::Primitive;

use super::{Valid, gene::Gene};

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
    type Gene;

    fn as_slice(&self) -> &[Self::Gene];
    fn as_mut_slice(&mut self) -> &mut [Self::Gene];

    fn get(&self, index: usize) -> Option<&Self::Gene> {
        self.as_slice().get(index)
    }

    fn get_mut(&mut self, index: usize) -> Option<&mut Self::Gene> {
        self.as_mut_slice().get_mut(index)
    }

    fn set(&mut self, index: usize, gene: Self::Gene) {
        self.as_mut_slice()[index] = gene;
    }

    fn len(&self) -> usize {
        self.as_slice().len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> impl Iterator<Item = &Self::Gene> {
        self.as_slice().iter()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Self::Gene> {
        self.as_mut_slice().iter_mut()
    }

    fn zip_mut<'a>(
        &'a mut self,
        other: &'a mut Self,
    ) -> impl Iterator<Item = (&'a mut Self::Gene, &'a mut Self::Gene)> {
        self.iter_mut().zip(other.iter_mut())
    }

    fn apply_paired<F>(&mut self, other: &mut Self, mut op: F)
    where
        Self: Sized,
        F: FnMut(&mut Self::Gene, &mut Self::Gene),
    {
        for (a, b) in self.iter_mut().zip(other.iter_mut()) {
            op(a, b);
        }
    }
}

pub trait NumericChromosome<T: Primitive>: Chromosome<Gene = T> {}
