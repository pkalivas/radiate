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

    fn genes(&self) -> &[Self::Gene];
    fn genes_mut(&mut self) -> &mut [Self::Gene];

    /// Retrieves the gene at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The position of the gene to retrieve.
    ///
    fn get(&self, index: usize) -> &Self::Gene {
        &self.genes()[index]
    }

    fn get_mut(&mut self, index: usize) -> &mut Self::Gene {
        &mut self.genes_mut()[index]
    }

    /// Sets the gene at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The position of the gene to set.
    /// * [`Gene`] - The gene to replace at the specified index.
    ///
    fn set(&mut self, index: usize, gene: Self::Gene) {
        self.genes_mut()[index] = gene;
    }

    fn len(&self) -> usize {
        self.genes().len()
    }

    fn iter(&self) -> std::slice::Iter<'_, Self::Gene> {
        self.genes().iter()
    }

    fn iter_mut(&mut self) -> std::slice::IterMut<'_, Self::Gene> {
        self.genes_mut().iter_mut()
    }
}
