use super::Valid;
use crate::Gene;

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

    fn zip_mut<'a>(&'a mut self, other: &'a mut Self) -> ZippedChromosome<'a, Self>
    where
        Self: Sized,
    {
        ZippedChromosome {
            chrom_one: self,
            chrom_two: other,
        }
    }
}

pub struct ZippedChromosome<'a, C: Chromosome> {
    chrom_one: &'a mut C,
    chrom_two: &'a mut C,
}

impl<'a, C: Chromosome> ZippedChromosome<'a, C> {
    pub fn iter(&'a mut self) -> impl Iterator<Item = (&'a mut C::Gene, &'a mut C::Gene)> {
        self.chrom_one.iter_mut().zip(self.chrom_two.iter_mut())
    }

    pub fn for_each<F>(&'a mut self, mut f: F)
    where
        F: FnMut(&'a mut C::Gene, &'a mut C::Gene),
    {
        for (gene_one, gene_two) in self.iter() {
            f(gene_one, gene_two);
        }
    }
}
