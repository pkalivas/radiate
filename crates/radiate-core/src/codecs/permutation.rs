use crate::{
    Chromosome, Codec, Gene, Genotype, PermutationChromosome, PermutationGene, random_provider,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct PermutationCodec<A: PartialEq + Clone> {
    alleles: Arc<[A]>,
}

impl<A: PartialEq + Clone> PermutationCodec<A> {
    pub fn new(alleles: Vec<A>) -> Self {
        PermutationCodec {
            alleles: alleles.into_boxed_slice().into(),
        }
    }
}

impl<A: PartialEq + Clone> Codec<PermutationChromosome<A>, Vec<A>> for PermutationCodec<A> {
    fn encode(&self) -> Genotype<PermutationChromosome<A>> {
        Genotype::from(PermutationChromosome::new(
            random_provider::shuffled_indices(0..self.alleles.len())
                .iter()
                .map(|i| PermutationGene::new(*i, Arc::clone(&self.alleles)))
                .collect(),
            Arc::clone(&self.alleles),
        ))
    }

    fn decode(&self, genotype: &Genotype<PermutationChromosome<A>>) -> Vec<A> {
        genotype
            .iter()
            .flat_map(|chromosome| chromosome.genes().iter().map(|gene| gene.allele().clone()))
            .collect()
    }
}

impl<A: PartialEq + Clone> Codec<PermutationChromosome<A>, Vec<Vec<A>>>
    for Vec<PermutationChromosome<A>>
{
    fn encode(&self) -> Genotype<PermutationChromosome<A>> {
        Genotype::from(
            self.iter()
                .map(|chromosome| {
                    PermutationChromosome::new(
                        chromosome
                            .genes()
                            .iter()
                            .map(|gene| gene.new_instance())
                            .collect(),
                        Arc::clone(chromosome.alleles()),
                    )
                })
                .collect::<Vec<PermutationChromosome<A>>>(),
        )
    }

    fn decode(&self, genotype: &Genotype<PermutationChromosome<A>>) -> Vec<Vec<A>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .genes()
                    .iter()
                    .map(|gene| gene.allele().clone())
                    .collect::<Vec<A>>()
            })
            .collect::<Vec<Vec<A>>>()
    }
}

impl<A: PartialEq + Clone> Codec<PermutationChromosome<A>, Vec<A>> for PermutationChromosome<A> {
    fn encode(&self) -> Genotype<PermutationChromosome<A>> {
        Genotype::from(PermutationChromosome::new(
            self.genes()
                .iter()
                .map(|gene| gene.new_instance())
                .collect(),
            Arc::clone(self.alleles()),
        ))
    }

    fn decode(&self, genotype: &Genotype<PermutationChromosome<A>>) -> Vec<A> {
        genotype
            .iter()
            .flat_map(|chromosome| chromosome.genes().iter().map(|gene| gene.allele().clone()))
            .collect()
    }
}
