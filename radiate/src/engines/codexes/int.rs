use super::Codex;
use crate::engines::genome::gene::Gene;
use crate::engines::genome::genotype::Genotype;
use crate::{Chromosome, IntChromosome, Integer};
use std::ops::Range;

/// A `Codex` for a `Genotype` of `IntGenes`. The `encode` function creates a `Genotype` with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `Vec<Vec<T>>` from the `Genotype` where the inner `Vec`
/// contains the alleles of the `IntGenes` in the chromosome. `T` must implement the `Integer` trait, meaning it must be one of
/// `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, or `u128`.
///
/// The lower and upper bounds of the `IntGenes` can be set with the `with_bounds` function.
/// The default bounds are equal to `min` and `max`.
#[derive(Clone)]
pub struct IntCodex<T: Integer<T>> {
    num_chromosomes: usize,
    num_genes: usize,
    value_range: Range<T>,
    bounds: Range<T>,
}

impl<T: Integer<T>> IntCodex<T> {
    pub fn new(num_chromosomes: usize, num_genes: usize, range: Range<T>) -> Self {
        IntCodex {
            num_chromosomes,
            num_genes,
            value_range: range.clone(),
            bounds: range,
        }
    }

    pub fn with_bounds(mut self, lower_bound: T, upper_bound: T) -> Self {
        self.bounds = lower_bound..upper_bound;
        self
    }
}

impl<T: Integer<T>> Codex<IntChromosome<T>, Vec<Vec<T>>> for IntCodex<T> {
    fn encode(&self) -> Genotype<IntChromosome<T>> {
        Genotype::from(
            (0..self.num_chromosomes)
                .map(|_| {
                    IntChromosome::from((
                        self.num_genes,
                        self.value_range.clone(),
                        self.bounds.clone(),
                    ))
                })
                .collect::<Vec<IntChromosome<T>>>(),
        )
    }

    fn decode(&self, genotype: &Genotype<IntChromosome<T>>) -> Vec<Vec<T>> {
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

impl<T: Integer<T>> Default for IntCodex<T> {
    fn default() -> Self {
        IntCodex {
            num_chromosomes: 1,
            num_genes: 1,
            value_range: T::MIN..T::MAX,
            bounds: T::MIN..T::MAX,
        }
    }
}
