use crate::{Chromosome, IntGene, Integer, Valid};
use rand::distributions::Standard;

/// A `Chromosome` that contains `IntGenes`.
/// 
#[derive(Clone, PartialEq)]
pub struct IntChromosome<I: Integer<I>>
where
    Standard: rand::distributions::Distribution<I>,
{
    pub genes: Vec<IntGene<I>>,
}

impl<I: Integer<I>> Chromosome for IntChromosome<I>
where
    Standard: rand::distributions::Distribution<I>,
{
    type GeneType = IntGene<I>;

    fn from_genes(genes: Vec<IntGene<I>>) -> Self {
        IntChromosome { genes }
    }

    fn set_gene(&mut self, index: usize, gene: IntGene<I>) {
        self.genes[index] = gene;
    }

    fn get_genes(&self) -> &[IntGene<I>] {
        &self.genes
    }

    fn get_genes_mut(&mut self) -> &mut [IntGene<I>] {
        &mut self.genes
    }
}

impl<T: Integer<T>> Valid for IntChromosome<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}
