use super::Codec;
use crate::genome::Gene;
use crate::genome::genotype::Genotype;
use crate::{Chromosome, IntChromosome, Integer};
use std::ops::Range;

/// A [Codec] for a [Genotype] of `IntGenes`. The `encode` function creates a [Genotype] with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `Vec<Vec<T>>` from the [Genotype] where the inner `Vec`
/// contains the alleles of the `IntGenes` in the chromosome. `T` must implement the `Integer` trait, meaning it must be one of
/// `i8`, `i16`, `i32`, `i64`, `i128`, `u8`, `u16`, `u32`, `u64`, or `u128`.
///
/// The lower and upper bounds of the `IntGenes` can be set with the `with_bounds` function.
/// The default bounds are equal to `min` and `max`.
#[derive(Clone)]
pub struct IntCodec<T: Integer<T>, D = T> {
    num_chromosomes: usize,
    num_genes: usize,
    value_range: Range<T>,
    bounds: Range<T>,
    _marker: std::marker::PhantomData<D>,
}

impl<T: Integer<T>, D> IntCodec<T, D> {
    pub fn with_bounds(mut self, bounds: Range<T>) -> Self {
        self.bounds = bounds;
        self
    }

    /// The different variants of `IntCodec` are all the same, so this function is used to create
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

impl<T: Integer<T>> IntCodec<T, Vec<Vec<T>>> {
    /// Create a new `IntCodec` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `IntGene` will be randomly generated between the min and max values.
    pub fn matrix(rows: usize, cols: usize, range: Range<T>) -> Self {
        IntCodec {
            num_chromosomes: rows,
            num_genes: cols,
            value_range: range.clone(),
            bounds: range,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Integer<T>> IntCodec<T, Vec<T>> {
    /// Create a new `IntCodec` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `IntGene` will be randomly generated between the min and max values.
    pub fn vector(count: usize, range: Range<T>) -> Self {
        IntCodec {
            num_chromosomes: 1,
            num_genes: count,
            value_range: range.clone(),
            bounds: range,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T: Integer<T>> IntCodec<T, T> {
    /// Create a new `IntCodec` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `IntGene` will be randomly generated between the min and max values.
    pub fn scalar(range: Range<T>) -> Self {
        IntCodec {
            num_chromosomes: 1,
            num_genes: 1,
            value_range: range.clone(),
            bounds: range,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Implement the [Codec] trait for a [Genotype] of `IntGenes`. This will produce a [Genotype] with the
/// given number of chromosomes and genes. The `decode` function will create a `Vec<Vec<T>>` or a matrix.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new IntCodec with 10 chromosomes with 10 genes
/// // per chromosome - a matrix of i32 values.
/// let codec = IntCodec::matrix(10, 10, 0..100);
/// let genotype: Genotype<IntChromosome<i32>> = codec.encode();
/// let decoded: Vec<Vec<i32>> = codec.decode(&genotype);
/// ```
impl<T: Integer<T>> Codec<IntChromosome<T>, Vec<Vec<T>>> for IntCodec<T, Vec<Vec<T>>> {
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

/// Implement the [Codec] trait for a [Genotype] of `IntGenes`. This will produce a [Genotype] with a single
/// chromosome and `num_genes` genes. The `decode` function will create a `Vec<T>` or a vector.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new IntCodec with 10 genes
/// // per chromosome - a  vector of i32 values.
/// let codec = IntCodec::vector(10, 0..100);
/// let genotype: Genotype<IntChromosome<i32>> = codec.encode();
/// let decoded: Vec<i32> = codec.decode(&genotype);
/// ```
impl<T: Integer<T>> Codec<IntChromosome<T>, Vec<T>> for IntCodec<T, Vec<T>> {
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

/// Implement the [Codec] trait for a [Genotype] of `IntGenes`. This will produce a [Genotype] with a single
/// chromosome and a single gene. The `decode` function will create a `T` or a single value.
/// The `encode` function creates a [Genotype] with a single chromosomes
/// and a single gene per chromosome.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new IntCodec with a single gene
/// // per chromosome - a single i32 value.
/// let codec = IntCodec::scalar(0..100);
/// let genotype: Genotype<IntChromosome<i32>> = codec.encode();
/// let decoded: i32 = codec.decode(&genotype);
/// ```
impl<T: Integer<T>> Codec<IntChromosome<T>, T> for IntCodec<T, T> {
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

impl<T: Integer<T>> Codec<IntChromosome<T>, Vec<Vec<T>>> for Vec<IntChromosome<T>> {
    fn encode(&self) -> Genotype<IntChromosome<T>> {
        Genotype::from(
            self.iter()
                .map(|chromosome| {
                    chromosome
                        .iter()
                        .map(|gene| gene.new_instance())
                        .collect::<IntChromosome<T>>()
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

impl<T: Integer<T>> Codec<IntChromosome<T>, Vec<T>> for IntChromosome<T> {
    fn encode(&self) -> Genotype<IntChromosome<T>> {
        Genotype::from(
            self.iter()
                .map(|gene| gene.new_instance())
                .collect::<IntChromosome<T>>(),
        )
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
