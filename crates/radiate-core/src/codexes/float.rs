use super::Codex;
use crate::genome::gene::Gene;
use crate::genome::genotype::Genotype;
use crate::{Chromosome, FloatChromosome};
use std::ops::Range;

/// A `Codex` for a `Genotype` of `FloatGenes`. The `encode` function creates a `Genotype` with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `Vec<Vec<f32>>` from the `Genotype` where the inner `Vec`
/// contains the alleles of the `FloatGenes` in the chromosome - the `f32` values.
///
/// The lower and upper bounds of the `FloatGenes` can be set with the `with_bounds` function.
/// The default bounds are equal to `min` and `max` values.
#[derive(Clone)]
pub struct FloatCodex<T = f32> {
    num_chromosomes: usize,
    num_genes: usize,
    value_range: Range<f32>,
    bounds: Range<f32>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> FloatCodex<T> {
    /// Set the bounds of the `FloatGenes` in the `Genotype`. The default bounds
    /// are equal to the min and max values.
    pub fn with_bounds(mut self, range: Range<f32>) -> Self {
        self.bounds = range;
        self
    }

    /// Every impl of `Codex` uses the same encode function for the `FloatCodex`, jsut with a few
    /// different parameters (e.g. `num_chromosomes` and `num_genes`). So, we can just use
    /// the same function for all of them.
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

impl FloatCodex<Vec<Vec<f32>>> {
    /// Create a new `FloatCodex` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `FloatGene` will be randomly generated between the min and max values.
    pub fn matrix(rows: usize, cols: usize, range: Range<f32>) -> Self {
        FloatCodex {
            num_chromosomes: rows,
            num_genes: cols,
            value_range: range.clone(),
            bounds: range,
            _marker: std::marker::PhantomData,
        }
    }
}

impl FloatCodex<Vec<f32>> {
    /// Create a new `FloatCodex` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `FloatGene` will be randomly generated between the min and max values.
    pub fn vector(count: usize, range: Range<f32>) -> Self {
        FloatCodex {
            num_chromosomes: 1,
            num_genes: count,
            value_range: range.clone(),
            bounds: range,
            _marker: std::marker::PhantomData,
        }
    }
}

impl FloatCodex<f32> {
    /// Create a new `FloatCodex` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `FloatGene` will be randomly generated between the min and max values.
    pub fn scalar(range: Range<f32>) -> Self {
        FloatCodex {
            num_chromosomes: 1,
            num_genes: 1,
            value_range: range.clone(),
            bounds: range,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Implement the `Codex` trait for a `FloatCodex` with a `Vec<Vec<f32>>` type.
/// This will decode to a matrix of `f32` values.
/// The `encode` function creates a `Genotype` with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome.
///
/// * Example:
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new FloatCodex with 3 chromosomes and 4 genes
/// // per chromosome - a 3x4 matrix of f32 values.
/// let codex = FloatCodex::matrix(3, 4, 0.0..1.0);
/// let genotype: Genotype<FloatChromosome> = codex.encode();
/// let decoded: Vec<Vec<f32>> = codex.decode(&genotype);
///
/// assert_eq!(decoded.len(), 3);
/// assert_eq!(decoded[0].len(), 4);
/// ```
impl Codex<FloatChromosome, Vec<Vec<f32>>> for FloatCodex<Vec<Vec<f32>>> {
    fn encode(&self) -> Genotype<FloatChromosome> {
        self.common_encode()
    }

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

/// Implement the `Codex` trait for a `FloatCodex` with a `Vec<f32>` type.
/// This will decode to a vector of `f32` values.
/// The `encode` function creates a `Genotype` with a single chromosomes
/// and `num_genes` genes per chromosome.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new FloatCodex with 3 genes
/// // per chromosome - a vector with 3 f32 values.
/// let codex = FloatCodex::vector(3, 0.0..1.0);
/// let genotype: Genotype<FloatChromosome> = codex.encode();
/// let decoded: Vec<f32> = codex.decode(&genotype);
///
/// assert_eq!(decoded.len(), 3);
/// ```
impl Codex<FloatChromosome, Vec<f32>> for FloatCodex<Vec<f32>> {
    fn encode(&self) -> Genotype<FloatChromosome> {
        self.common_encode()
    }

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

/// Implement the `Codex` trait for a `FloatCodex` with a `f32` type.
/// This will decode to a single `f32` value.
/// The `encode` function creates a `Genotype` with a single chromosomes
/// and a single gene per chromosome.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new FloatCodex with a single gene
/// // per chromosome - a single f32 value.
/// let codex = FloatCodex::scalar(0.0..1.0);
/// let genotype: Genotype<FloatChromosome> = codex.encode();
/// let decoded: f32 = codex.decode(&genotype);
/// ```
impl Codex<FloatChromosome, f32> for FloatCodex<f32> {
    fn encode(&self) -> Genotype<FloatChromosome> {
        self.common_encode()
    }

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
