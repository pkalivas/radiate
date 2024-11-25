use rand::distributions::Standard;

use crate::engines::genome::chromosome::Chromosome;
use crate::engines::genome::genes::gene::{BoundGene, Gene};
use crate::engines::genome::genes::int_gene::IntGene;
use crate::engines::genome::genotype::Genotype;
use crate::Integer;

use super::Codex;

/// A `Codex` for a `Genotype` of `IntGenes`. The `encode` function creates a `Genotype` with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `Vec<Vec<T>>` from the `Genotype` where the inner `Vec`
/// contains the alleles of the `IntGenes` in the chromosome. `T` must implement the `Integer` trait, meaning it must be one of
/// `i8`, `i16`, `i32`, `i64`, or `i128`.
///
/// The lower and upper bounds of the `IntGenes` can be set with the `with_bounds` function.
/// The default bounds are `T::MIN` and `T::MAX`.
pub struct IntCodex<T: Integer<T>>
where
    Standard: rand::distributions::Distribution<T>,
{
    pub num_chromosomes: usize,
    pub num_genes: usize,
    pub min: T,
    pub max: T,
    pub lower_bound: T,
    pub upper_bound: T,
}

impl<T: Integer<T>> IntCodex<T>
where
    Standard: rand::distributions::Distribution<T>,
{
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

impl<T: Integer<T>> Codex<IntGene<T>, T, Vec<Vec<T>>> for IntCodex<T>
where
    Standard: rand::distributions::Distribution<T>,
{
    fn encode(&self) -> Genotype<IntGene<T>, T> {
        Genotype {
            chromosomes: (0..self.num_chromosomes)
                .map(|_| {
                    Chromosome::from_genes(
                        (0..self.num_genes)
                            .map(|_| {
                                IntGene::from_min_max(self.min, self.max)
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
