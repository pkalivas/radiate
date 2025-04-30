use crate::{Codex, Gene, Genotype, PermutationChromosome, PermutationGene, random_provider};
use std::sync::Arc;

#[derive(Clone)]
pub struct PermutationCodex<A: PartialEq + Clone> {
    alleles: Arc<Vec<A>>,
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
        Genotype::new(vec![PermutationChromosome {
            genes: random_provider::indexes(0..self.alleles.len())
                .iter()
                .map(|i| PermutationGene::new(*i, Arc::clone(&self.alleles)))
                .collect(),
            alleles: Arc::clone(&self.alleles),
        }])
    }

    fn decode(&self, genotype: &Genotype<PermutationChromosome<A>>) -> Vec<A> {
        genotype
            .iter()
            .flat_map(|chromosome| chromosome.genes.iter().map(|gene| gene.allele().clone()))
            .collect()
    }
}
