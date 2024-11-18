use super::genes::gene::Gene;

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
