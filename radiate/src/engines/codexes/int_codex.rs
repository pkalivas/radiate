use crate::engines::genome::chromosome::Chromosome;
use crate::engines::genome::genes::gene::{BoundGene, Gene};
use crate::engines::genome::genes::int_gene::IntGene;
use crate::engines::genome::genotype::Genotype;
use crate::Integer;

use super::Codex;

pub struct IntCodex<T: Integer<T>> {
    pub num_chromosomes: usize,
    pub num_genes: usize,
    pub min: T,
    pub max: T,
    pub lower_bound: T,
    pub upper_bound: T,
}

impl<T: Integer<T>> IntCodex<T> {
    pub fn new(num_chromosomes: usize, num_genes: usize, min: T, max: T) -> Self {
        IntCodex {
            num_chromosomes,
            num_genes,
            min,
            max,
            lower_bound: T::MIN,
            upper_bound: T::MAX,
        }
    }

    pub fn with_bounds(mut self, lower_bound: T, upper_bound: T) -> Self {
        self.lower_bound = lower_bound;
        self.upper_bound = upper_bound;
        self
    }
}

impl<T: Integer<T>> Codex<IntGene<T>, T, Vec<Vec<T>>> for IntCodex<T> {
    fn encode(&self) -> Genotype<IntGene<T>, T> {
        Genotype {
            chromosomes: (0..self.num_chromosomes)
                .into_iter()
                .map(|_| {
                    Chromosome::from_genes(
                        (0..self.num_genes)
                            .into_iter()
                            .map(|_| {
                                IntGene::new(self.min, self.max)
                                    .with_bounds(self.lower_bound, self.upper_bound)
                            })
                            .collect::<Vec<IntGene<T>>>(),
                    )
                })
                .collect::<Vec<Chromosome<IntGene<T>, T>>>(),
        }
    }

    fn decode(&self, genotype: &Genotype<IntGene<T>, T>) -> Vec<Vec<T>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<T>>()
            })
            .collect::<Vec<Vec<T>>>()
    }
}
