use super::Codex;
use crate::genome::gene::Gene;
use crate::genome::genotype::Genotype;
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
pub struct IntCodex<T: Integer<T>, D = T> {
    num_chromosomes: usize,
    num_genes: usize,
    value_range: Range<T>,
    bounds: Range<T>,
    _marker: std::marker::PhantomData<D>,
}

impl<T: Integer<T>, D> IntCodex<T, D> {
    pub fn with_bounds(mut self, bounds: Range<T>) -> Self {
        self.bounds = bounds;
        self
    }

    /// The different variants of `IntCodex` are all the same, so this function is used to create
    /// a new `Genotype` with the given number of chromosomes and genes. The only difference between
    /// them is the type `D`, which is either a `Vec<Vec<T>>`, `Vec<T>`, or `T`.
    fn encode_common(&self) -> Genotype<IntChromosome<T>> {
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
}

impl<T: Integer<T>> IntCodex<T, Vec<Vec<T>>> {
    /// Create a new `IntCodex` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `IntGene` will be randomly generated between the min and max values.
    pub fn matrix(rows: usize, cols: usize, range: Range<T>) -> Self {
        IntCodex {
            num_chromosomes: rows,
            num_genes: cols,
            value_range: range.clone(),
            bounds: range,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Integer<T>> IntCodex<T, Vec<T>> {
    /// Create a new `IntCodex` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `IntGene` will be randomly generated between the min and max values.
    pub fn vector(count: usize, range: Range<T>) -> Self {
        IntCodex {
            num_chromosomes: 1,
            num_genes: count,
            value_range: range.clone(),
            bounds: range,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Integer<T>> IntCodex<T, T> {
    /// Create a new `IntCodex` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `IntGene` will be randomly generated between the min and max values.
    pub fn scalar(range: Range<T>) -> Self {
        IntCodex {
            num_chromosomes: 1,
            num_genes: 1,
            value_range: range.clone(),
            bounds: range,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Implement the `Codex` trait for a `Genotype` of `IntGenes`. This will produce a `Genotype` with the given number of chromosomes
/// and genes. The `decode` function will create a `Vec<Vec<T>>` or a matrix.
impl<T: Integer<T>> Codex<IntChromosome<T>, Vec<Vec<T>>> for IntCodex<T, Vec<Vec<T>>> {
    fn encode(&self) -> Genotype<IntChromosome<T>> {
        self.encode_common()
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

/// Implement the `Codex` trait for a `Genotype` of `IntGenes`. This will produce a `Genotype` with a single
/// chromosome and `num_genes` genes. The `decode` function will create a `Vec<T>` or a vector.
impl<T: Integer<T>> Codex<IntChromosome<T>, Vec<T>> for IntCodex<T, Vec<T>> {
    fn encode(&self) -> Genotype<IntChromosome<T>> {
        self.encode_common()
    }

    fn decode(&self, genotype: &Genotype<IntChromosome<T>>) -> Vec<T> {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<T>>()
            })
            .collect::<Vec<T>>()
    }
}

/// Implement the `Codex` trait for a `Genotype` of `IntGenes`. This will produce a `Genotype` with a single
/// chromosome and a single gene. The `decode` function will create a `T` or a single value.
impl<T: Integer<T>> Codex<IntChromosome<T>, T> for IntCodex<T, T> {
    fn encode(&self) -> Genotype<IntChromosome<T>> {
        self.encode_common()
    }

    fn decode(&self, genotype: &Genotype<IntChromosome<T>>) -> T {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<T>>()
            })
            .next()
            .unwrap_or_default()
    }
}
