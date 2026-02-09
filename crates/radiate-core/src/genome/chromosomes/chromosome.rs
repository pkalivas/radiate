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
    type Gene: Gene;

    fn as_slice(&self) -> &[Self::Gene];
    fn as_mut_slice(&mut self) -> &mut [Self::Gene];

    /// Retrieves the gene at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The position of the gene to retrieve.
    ///
    fn get(&self, index: usize) -> &Self::Gene {
        &self.as_slice()[index]
    }

    fn get_mut(&mut self, index: usize) -> &mut Self::Gene {
        &mut self.as_mut_slice()[index]
    }

    /// Sets the gene at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The position of the gene to set.
    /// * [`Gene`] - The gene to replace at the specified index.
    ///
    fn set(&mut self, index: usize, gene: Self::Gene) {
        self.as_mut_slice()[index] = gene;
    }

    fn len(&self) -> usize {
        self.as_slice().len()
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
}
