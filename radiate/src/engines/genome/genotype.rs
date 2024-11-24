use super::{chromosome::Chromosome, genes::gene::Gene};

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
/// - `G`: The type of gene used in the genetic algorithm, which must implement the `Gene` trait.
/// - `A`: The type of the allele associated with the gene - the gene's "expression".
///
pub struct Genotype<G, A>
where
    G: Gene<G, A>,
{
    pub chromosomes: Vec<Chromosome<G, A>>,
}

impl<G, A> Genotype<G, A>
where
    G: Gene<G, A>,
{
    /// Create a new instance of the Genotype with the given chromosomes.
    pub fn from_chromosomes(chromosomes: Vec<Chromosome<G, A>>) -> Self {
        Genotype { chromosomes }
    }

    pub fn get_chromosome_mut(&mut self, index: usize) -> &mut Chromosome<G, A> {
        &mut self.chromosomes[index]
    }

    pub fn get_chromosome(&self, index: usize) -> &Chromosome<G, A> {
        &self.chromosomes[index]
    }

    pub fn set_chromosome(&mut self, index: usize, chromosome: Chromosome<G, A>) {
        self.chromosomes[index] = chromosome;
    }

    pub fn len(&self) -> usize {
        self.chromosomes.len()
    }

    pub fn is_valid(&self) -> bool {
        self.chromosomes
            .iter()
            .all(|chromosome| chromosome.is_valid())
    }

    pub fn iter(&self) -> std::slice::Iter<Chromosome<G, A>> {
        self.chromosomes.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<Chromosome<G, A>> {
        self.chromosomes.iter_mut()
    }
}

impl<G, A> Clone for Genotype<G, A>
where
    G: Gene<G, A>,
{
    fn clone(&self) -> Self {
        Genotype {
            chromosomes: self.chromosomes.clone(),
        }
    }
}

impl<G, A> PartialEq for Genotype<G, A>
where
    G: Gene<G, A>,
{
    fn eq(&self, other: &Self) -> bool {
        self.chromosomes == other.chromosomes
    }
}

impl<G, A> Into<Genotype<G, A>> for Vec<Chromosome<G, A>>
where
    G: Gene<G, A>,
{
    fn into(self) -> Genotype<G, A> {
        Genotype { chromosomes: self }
    }
}

impl<G, A> Into<Vec<Chromosome<G, A>>> for Genotype<G, A>
where
    G: Gene<G, A>,
{
    fn into(self) -> Vec<Chromosome<G, A>> {
        self.chromosomes
    }
}

impl<G, A> std::iter::FromIterator<Chromosome<G, A>> for Genotype<G, A>
where
    G: Gene<G, A>,
{
    fn from_iter<I: IntoIterator<Item = Chromosome<G, A>>>(iter: I) -> Self {
        Genotype {
            chromosomes: iter.into_iter().collect(),
        }
    }
}

impl<G, A> std::fmt::Debug for Genotype<G, A>
where
    G: Gene<G, A> + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for chromosome in &self.chromosomes {
            write!(f, "{:?},\n ", chromosome)?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::engines::genome::genes::float_gene::FloatGene;

    #[test]
    fn test_new() {
        let genotype = Genotype::from_chromosomes(vec![Chromosome::from_genes(vec![
            FloatGene::new(0_f32, 1_f32),
            FloatGene::new(0_f32, 1_f32),
        ])]);

        assert!(genotype.is_valid());
    }

    #[test]
    fn test_into() {
        let genotype = Genotype::from_chromosomes(vec![Chromosome::from_genes(vec![
            FloatGene::new(0_f32, 1_f32),
            FloatGene::new(0_f32, 1_f32),
        ])]);

        let chromosomes: Vec<Chromosome<FloatGene, f32>> = genotype.into();
        assert_eq!(chromosomes.len(), 1);
    }

    #[test]
    fn test_from_iter() {
        let chromosomes = vec![Chromosome::from_genes(vec![
            FloatGene::new(0_f32, 1_f32),
            FloatGene::new(0_f32, 1_f32),
        ])];
        let genotype: Genotype<FloatGene, f32> = chromosomes.into_iter().collect();
        assert_eq!(genotype.len(), 1);
    }
}
