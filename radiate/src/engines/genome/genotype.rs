use crate::{Chromosome, Valid};

/// The `Genotype` struct represents the genetic makeup of an individual. It is a collection of `Chromosome` instances, it is
/// essentially a light wrapper around a Vec of `Chromosome`s. The `Genotype` struct, however, has some additional functionality
/// and terminology that aligns with the biological concept of a genotype.
/// In traditional biological terms, a `Genotype` is the set of genes in our DNA that determine a specific trait or set of traits.
/// The `Genotype` is the 'genetic' part of the individual that is being evolved by the genetic algorithm.
///
/// We can think of a `Genotype`  as a matrix of strucs which implement the `Gene` trait where each row is a `Chromosome`.
/// For example, if we have a `Genotype` with 2 `Chromosome`s, each with 3 `Gene`s, it is represented as follows:
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
#[derive(Clone, PartialEq, Debug)]
pub struct Genotype<C: Chromosome> {
    pub chromosomes: Vec<C>,
}

impl<C: Chromosome> Genotype<C> {
    pub fn from_chromosomes(chromosomes: Vec<C>) -> Self {
        Genotype { chromosomes }
    }

    pub fn get_chromosome_mut(&mut self, index: usize) -> &mut C {
        &mut self.chromosomes[index]
    }

    pub fn get_chromosome(&self, index: usize) -> &C {
        &self.chromosomes[index]
    }

    pub fn set_chromosome(&mut self, index: usize, chromosome: C) {
        self.chromosomes[index] = chromosome;
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
}

impl<C: Chromosome> Valid for Genotype<C> {
    fn is_valid(&self) -> bool {
        self.chromosomes
            .iter()
            .all(|chromosome| chromosome.is_valid())
    }
}
