use std::{sync::Arc, vec};

use crate::{PermutationGene, Valid};

use super::Chromosome;

#[derive(Debug, Clone, PartialEq)]
pub struct PermutationChromosome<A: PartialEq + Clone> {
    pub genes: Vec<PermutationGene<A>>,
    pub alleles: Arc<Vec<A>>,
}

impl<A: PartialEq + Clone> PermutationChromosome<A> {
    pub fn new(genes: Vec<PermutationGene<A>>, alleles: Arc<Vec<A>>) -> Self {
        PermutationChromosome { genes, alleles }
    }
}

impl<A: PartialEq + Clone> Chromosome for PermutationChromosome<A> {
    type GeneType = PermutationGene<A>;

    fn from_genes(genes: Vec<PermutationGene<A>>) -> Self {
        let alleles = match genes.first() {
            Some(gene) => Arc::clone(&gene.alleles),
            None => Arc::new(Vec::new()),
        };

        PermutationChromosome { genes, alleles }
    }

    fn set_gene(&mut self, index: usize, gene: PermutationGene<A>) {
        self.genes[index] = gene;
    }

    fn get_genes(&self) -> &[PermutationGene<A>] {
        &self.genes
    }

    fn get_genes_mut(&mut self) -> &mut [PermutationGene<A>] {
        &mut self.genes
    }
}

impl<A: PartialEq + Clone> Valid for PermutationChromosome<A> {
    fn is_valid(&self) -> bool {
        // Check if the genes are a valid permutation of the alleles
        let mut bit_set = vec![false; self.alleles.len()];
        self.genes.iter().all(|gene| {
            let index = gene.index;
            if bit_set[index] {
                return false;
            }
            bit_set[index] = true;
            true
        })
    }
}
