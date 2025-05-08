use crate::{Chromosome, Valid};
use std::ops::{Index, IndexMut};

/// The [Genotype] struct represents the genetic makeup of an individual. It is a collection of [Chromosome] instances, it is
/// essentially a light wrapper around a Vec of [Chromosome]s. The [Genotype] struct, however, has some additional functionality
/// and terminology that aligns with the biological concept of a genotype.
/// In traditional biological terms, a [Genotype] is the set of genes in our DNA that determine a specific trait or set of traits.
/// The [Genotype] is the 'genetic' part of the individual that is being evolved by the genetic algorithm.
///
/// We can think of a [Genotype]  as a matrix of strucs which implement the `Gene` trait where each row is a [Chromosome].
/// For example, if we have a [Genotype] with 2 [Chromosome]s, each with 3 `Gene`s, it is represented as follows:
/// ```text
/// Genotype:
/// [
///     Chromosome: [Gene, Gene, Gene],
///     Chromosome: [Gene, Gene, Gene]
/// ]
/// ```
///
/// # Type Parameters
/// - `C`: The type of chromosome used in the genotype, which must implement the `Chromosome` trait.
///
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Genotype<C: Chromosome> {
    chromosomes: Vec<C>,
}

impl<C: Chromosome> Genotype<C> {
    pub fn new(chromosomes: Vec<C>) -> Self {
        Genotype { chromosomes }
    }

    pub fn len(&self) -> usize {
        self.chromosomes.len()
    }

    pub fn iter(&self) -> std::slice::Iter<C> {
        self.chromosomes.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<C> {
        self.chromosomes.iter_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.chromosomes.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&C> {
        self.chromosomes.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut C> {
        self.chromosomes.get_mut(index)
    }

    pub fn take_chromosomes(&mut self) -> Vec<C> {
        std::mem::take(&mut self.chromosomes)
    }
}

impl<C: Chromosome> Valid for Genotype<C> {
    fn is_valid(&self) -> bool {
        self.chromosomes
            .iter()
            .all(|chromosome| chromosome.is_valid())
    }
}

impl<C: Chromosome> AsRef<[C]> for Genotype<C> {
    fn as_ref(&self) -> &[C] {
        &self.chromosomes
    }
}

impl<C: Chromosome> Index<usize> for Genotype<C> {
    type Output = C;

    fn index(&self, index: usize) -> &Self::Output {
        &self.chromosomes[index]
    }
}

impl<C: Chromosome> IndexMut<usize> for Genotype<C> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.chromosomes[index]
    }
}

impl<C: Chromosome> From<Vec<C>> for Genotype<C> {
    fn from(chromosomes: Vec<C>) -> Self {
        Genotype { chromosomes }
    }
}

unsafe impl<C: Chromosome> Send for Genotype<C> {}
unsafe impl<C: Chromosome> Sync for Genotype<C> {}
