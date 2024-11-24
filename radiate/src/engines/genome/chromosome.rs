use super::genes::gene::Gene;

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
/// # Type Parameters
/// - `G`: The type of gene used in the genetic algorithm, which must implement the `Gene` trait.
/// - `A`: The type of the allele associated with the gene - the gene's "expression".
///
pub struct Chromosome<G, A>
where
    G: Gene<G, A>,
{
    pub genes: Vec<G>,
    _allele: std::marker::PhantomData<A>,
}

impl<G, A> Chromosome<G, A>
where
    G: Gene<G, A>,
{
    /// Create a new instance of the Chromosome with the given genes.
    pub fn from_genes(genes: Vec<G>) -> Self {
        Chromosome {
            genes,
            _allele: std::marker::PhantomData,
        }
    }

    pub fn get_gene(&self, index: usize) -> &G {
        &self.genes[index]
    }

    pub fn set_gene(&mut self, index: usize, gene: G) {
        self.genes[index] = gene;
    }

    pub fn get_genes(&self) -> &[G] {
        &self.genes
    }

    pub fn len(&self) -> usize {
        self.genes.len()
    }

    pub fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }

    pub fn iter(&self) -> std::slice::Iter<G> {
        self.genes.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<G> {
        self.genes.iter_mut()
    }
}

impl<G, A> Clone for Chromosome<G, A>
where
    G: Gene<G, A>,
{
    fn clone(&self) -> Self {
        Chromosome {
            genes: self.genes.clone(),
            _allele: std::marker::PhantomData,
        }
    }
}

impl<G, A> PartialEq for Chromosome<G, A>
where
    G: Gene<G, A>,
{
    fn eq(&self, other: &Self) -> bool {
        for (a, b) in self.genes.iter().zip(other.genes.iter()) {
            if a != b {
                return false;
            }
        }

        true
    }
}

impl<G, A> Into<Chromosome<G, A>> for Vec<G>
where
    G: Gene<G, A>,
{
    fn into(self) -> Chromosome<G, A> {
        Chromosome {
            genes: self,
            _allele: std::marker::PhantomData,
        }
    }
}

impl<G, A> Into<Vec<G>> for Chromosome<G, A>
where
    G: Gene<G, A>,
{
    fn into(self) -> Vec<G> {
        self.genes
    }
}

impl<G, A> std::iter::FromIterator<G> for Chromosome<G, A>
where
    G: Gene<G, A>,
{
    fn from_iter<I: IntoIterator<Item = G>>(iter: I) -> Self {
        Chromosome {
            genes: iter.into_iter().collect(),
            _allele: std::marker::PhantomData,
        }
    }
}

impl<G, A> std::fmt::Debug for Chromosome<G, A>
where
    G: Gene<G, A> + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for gene in &self.genes {
            write!(f, "{:?}, ", gene)?;
        }
        write!(f, "]")
    }
}

#[cfg(test)]
mod test {

    use super::super::genes::int_gene::IntGene;
    use super::*;

    #[test]
    fn test_from_genes() {
        let genes = vec![IntGene::new(0), IntGene::new(1)];
        let chromosome = Chromosome::from_genes(genes.clone());
        assert_eq!(chromosome.genes, genes);
    }
}
