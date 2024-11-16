use std::sync::Arc;

use crate::{Chromosome, Gene, GenericGene, Genotype};

use super::Codex;

pub struct GenericCodex<A>
where
    A: Clone + PartialEq,
{
    pub num_chromosomes: usize,
    pub num_genes: usize,
    pub supplier: Arc<dyn Fn() -> A>,
}

impl<A> GenericCodex<A>
where
    A: Clone + PartialEq,
{
    pub fn new(num_chromosomes: usize, num_genes: usize, supplier: Arc<dyn Fn() -> A>) -> Self {
        Self {
            num_chromosomes,
            num_genes,
            supplier,
        }
    }
}

impl<A> Codex<GenericGene<A>, A, Vec<Vec<A>>> for GenericCodex<A>
where
    A: Clone + PartialEq,
{
    fn encode(&self) -> Genotype<GenericGene<A>, A> {
        Genotype {
            chromosomes: (0..self.num_chromosomes)
                .into_iter()
                .map(|_| {
                    Chromosome::from_genes(
                        (0..self.num_genes)
                            .into_iter()
                            .map(|_| {
                                GenericGene::new((self.supplier)(), Arc::clone(&self.supplier))
                            })
                            .collect::<Vec<GenericGene<A>>>(),
                    )
                })
                .collect::<Vec<Chromosome<GenericGene<A>, A>>>(),
        }
    }

    fn decode(&self, genotype: &Genotype<GenericGene<A>, A>) -> Vec<Vec<A>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| gene.allele().clone())
                    .collect::<Vec<A>>()
            })
            .collect::<Vec<Vec<A>>>()
    }
}
