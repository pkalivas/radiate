use crate::{random_provider, Codex, Gene, Genotype, PermutationChromosome, PermutationGene};
use std::sync::Arc;

pub struct PermutationCodex<A: PartialEq + Clone> {
    pub alleles: Arc<Vec<A>>,
}

impl<A: PartialEq + Clone> PermutationCodex<A> {
    pub fn new(alleles: Vec<A>) -> Self {
        PermutationCodex {
            alleles: Arc::new(alleles),
        }
    }
}

impl<A: PartialEq + Clone> Codex<PermutationChromosome<A>, Vec<A>> for PermutationCodex<A> {
    fn encode(&self) -> Genotype<PermutationChromosome<A>> {
        let mut random_indexes: Vec<usize> = (0..self.alleles.len()).collect();
        random_provider::shuffle(&mut random_indexes);
        let genes = random_indexes
            .iter()
            .map(|i| PermutationGene::new(*i, Arc::clone(&self.alleles)))
            .collect();
        let chromosome = PermutationChromosome::new(genes, Arc::clone(&self.alleles));
        Genotype::from_chromosomes(vec![chromosome])
    }

    fn decode(&self, genotype: &Genotype<PermutationChromosome<A>>) -> Vec<A> {
        genotype
            .chromosomes
            .iter()
            .flat_map(|chromosome| chromosome.genes.iter().map(|gene| gene.allele().clone()))
            .collect()
    }
}
