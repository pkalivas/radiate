use super::Codec;
use crate::genome::Gene;
use crate::genome::genotype::Genotype;
use crate::{Chromosome, FloatChromosome};
use std::ops::Range;

/// A [Codec] for a [Genotype] of `FloatGenes`. The `encode` function creates a [Genotype] with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `Vec<Vec<f32>>` from the [Genotype] where the inner `Vec`
/// contains the alleles of the `FloatGenes` in the chromosome - the `f32` values.
///
/// The lower and upper bounds of the `FloatGenes` can be set with the `with_bounds` function.
/// The default bounds are equal to `min` and `max` values.
#[derive(Clone)]
pub struct FloatCodec<T = f32> {
    num_chromosomes: usize,
    num_genes: usize,
    value_range: Range<f32>,
    bounds: Range<f32>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> FloatCodec<T> {
    /// Set the bounds of the `FloatGenes` in the [Genotype]. The default bounds
    /// are equal to the min and max values.
    pub fn with_bounds(mut self, range: Range<f32>) -> Self {
        self.bounds = range;
        self
    }

    /// Every impl of `Codec` uses the same encode function for the `FloatCodec`, just with a few
    /// different parameters (e.g. `num_chromosomes` and `num_genes`). So, we can just use
    /// the same function for all of them.
    #[inline]
    fn common_encode(&self) -> Genotype<FloatChromosome> {
        Genotype::from(
            (0..self.num_chromosomes)
                .map(|_| {
                    FloatChromosome::from((
                        self.num_genes,
                        self.value_range.clone(),
                        self.bounds.clone(),
                    ))
                })
                .collect::<Vec<FloatChromosome>>(),
        )
    }
}

impl FloatCodec<Vec<Vec<f32>>> {
    /// Create a new `FloatCodec` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `FloatGene` will be randomly generated between the min and max values.
    pub fn matrix(rows: usize, cols: usize, range: Range<f32>) -> Self {
        FloatCodec {
            num_chromosomes: rows,
            num_genes: cols,
            value_range: range.clone(),
            bounds: range,
            _marker: std::marker::PhantomData,
        }
    }
}

impl FloatCodec<Vec<f32>> {
    /// Create a new `FloatCodec` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `FloatGene` will be randomly generated between the min and max values.
    pub fn vector(count: usize, range: Range<f32>) -> Self {
        FloatCodec {
            num_chromosomes: 1,
            num_genes: count,
            value_range: range.clone(),
            bounds: range,
            _marker: std::marker::PhantomData,
        }
    }
}

impl FloatCodec<f32> {
    /// Create a new `FloatCodec` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `FloatGene` will be randomly generated between the min and max values.
    pub fn scalar(range: Range<f32>) -> Self {
        FloatCodec {
            num_chromosomes: 1,
            num_genes: 1,
            value_range: range.clone(),
            bounds: range,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Implement the `Codec` trait for a `FloatCodec` with a `Vec<Vec<f32>>` type.
/// This will decode to a matrix of `f32` values.
/// The `encode` function creates a [Genotype] with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome.
///
/// * Example:
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new FloatCodec with 3 chromosomes and 4 genes
/// // per chromosome - a 3x4 matrix of f32 values.
/// let codec = FloatCodec::matrix(3, 4, 0.0..1.0);
/// let genotype: Genotype<FloatChromosome> = codec.encode();
/// let decoded: Vec<Vec<f32>> = codec.decode(&genotype);
///
/// assert_eq!(decoded.len(), 3);
/// assert_eq!(decoded[0].len(), 4);
/// ```
impl Codec<FloatChromosome, Vec<Vec<f32>>> for FloatCodec<Vec<Vec<f32>>> {
    #[inline]
    fn encode(&self) -> Genotype<FloatChromosome> {
        self.common_encode()
    }

    #[inline]
    fn decode(&self, genotype: &Genotype<FloatChromosome>) -> Vec<Vec<f32>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<f32>>()
            })
            .collect::<Vec<Vec<f32>>>()
    }
}

/// Implement the `Codec` trait for a `FloatCodec` with a `Vec<f32>` type.
/// This will decode to a vector of `f32` values.
/// The `encode` function creates a [Genotype] with a single chromosomes
/// and `num_genes` genes per chromosome.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new FloatCodec with 3 genes
/// // per chromosome - a vector with 3 f32 values.
/// let codec = FloatCodec::vector(3, 0.0..1.0);
/// let genotype: Genotype<FloatChromosome> = codec.encode();
/// let decoded: Vec<f32> = codec.decode(&genotype);
///
/// assert_eq!(decoded.len(), 3);
/// ```
impl Codec<FloatChromosome, Vec<f32>> for FloatCodec<Vec<f32>> {
    #[inline]
    fn encode(&self) -> Genotype<FloatChromosome> {
        self.common_encode()
    }

    #[inline]
    fn decode(&self, genotype: &Genotype<FloatChromosome>) -> Vec<f32> {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<f32>>()
            })
            .collect::<Vec<f32>>()
    }
}

/// Implement the `Codec` trait for a `FloatCodec` with a `f32` type.
/// This will decode to a single `f32` value.
/// The `encode` function creates a [Genotype] with a single chromosomes
/// and a single gene per chromosome.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new FloatCodec with a single gene
/// // per chromosome - a single f32 value.
/// let codec = FloatCodec::scalar(0.0..1.0);
/// let genotype: Genotype<FloatChromosome> = codec.encode();
/// let decoded: f32 = codec.decode(&genotype);
/// ```
impl Codec<FloatChromosome, f32> for FloatCodec<f32> {
    #[inline]
    fn encode(&self) -> Genotype<FloatChromosome> {
        self.common_encode()
    }

    #[inline]
    fn decode(&self, genotype: &Genotype<FloatChromosome>) -> f32 {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<f32>>()
            })
            .next()
            .unwrap_or_default()
    }
}

/// Implement the [Codec] trait for a Vec for [FloatChromosome].
/// This is effectively the same as creating a [FloatCodec] matrix
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// let codec = vec![
///     FloatChromosome::from((3, 0.0..1.0)),
///     FloatChromosome::from((4, 0.0..1.0)),
/// ];
///
/// let genotype: Genotype<FloatChromosome> = codec.encode();
/// let decoded: Vec<Vec<f32>> = codec.decode(&genotype);
///
/// assert_eq!(decoded.len(), 2);
/// assert_eq!(decoded[0].len(), 3);
/// assert_eq!(decoded[1].len(), 4);
/// ```
impl Codec<FloatChromosome, Vec<Vec<f32>>> for Vec<FloatChromosome> {
    #[inline]
    fn encode(&self) -> Genotype<FloatChromosome> {
        Genotype::from(
            self.iter()
                .map(|chromosome| {
                    chromosome
                        .iter()
                        .map(|gene| gene.new_instance())
                        .collect::<FloatChromosome>()
                })
                .collect::<Vec<FloatChromosome>>(),
        )
    }

    #[inline]
    fn decode(&self, genotype: &Genotype<FloatChromosome>) -> Vec<Vec<f32>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<f32>>()
            })
            .collect::<Vec<Vec<f32>>>()
    }
}

/// Implement the [Codec] trait for a single [FloatChromosome].
/// This is effectively the same as creating a [FloatCodec] vector
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// let codec = FloatChromosome::from((3, 0.0..1.0));
/// let genotype: Genotype<FloatChromosome> = codec.encode();
/// let decoded: Vec<f32> = codec.decode(&genotype);
///
/// assert_eq!(decoded.len(), 3);
/// ```
impl Codec<FloatChromosome, Vec<f32>> for FloatChromosome {
    #[inline]
    fn encode(&self) -> Genotype<FloatChromosome> {
        Genotype::from(
            self.iter()
                .map(|gene| gene.new_instance())
                .collect::<FloatChromosome>(),
        )
    }

    #[inline]
    fn decode(&self, genotype: &Genotype<FloatChromosome>) -> Vec<f32> {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<f32>>()
            })
            .collect::<Vec<f32>>()
    }
}
