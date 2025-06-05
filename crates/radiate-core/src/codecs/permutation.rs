use crate::{
    Chromosome, Codec, Gene, Genotype, PermutationChromosome, PermutationGene, random_provider,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct PermutationCodec<A: PartialEq + Clone> {
    alleles: Arc<Vec<A>>,
}

impl<A: PartialEq + Clone> PermutationCodec<A> {
    pub fn new(alleles: Vec<A>) -> Self {
        PermutationCodec {
            alleles: Arc::new(alleles),
        }
    }
}

impl<A: PartialEq + Clone> Codec<PermutationChromosome<A>, Vec<A>> for PermutationCodec<A> {
    fn encode(&self) -> Genotype<PermutationChromosome<A>> {
        Genotype::new(vec![PermutationChromosome::new(
            random_provider::indexes(0..self.alleles.len())
                .iter()
                .map(|i| PermutationGene::new(*i, Arc::clone(&self.alleles)))
                .collect(),
            Arc::clone(&self.alleles),
        )])
    }

    fn decode(&self, genotype: &Genotype<PermutationChromosome<A>>) -> Vec<A> {
        genotype
            .iter()
            .flat_map(|chromosome| chromosome.genes().iter().map(|gene| gene.allele().clone()))
            .collect()
    }
}
