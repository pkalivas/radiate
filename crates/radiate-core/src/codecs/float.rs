use radiate_utils::Float;

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
pub struct FloatCodec<F: Float, T = F> {
    num_chromosomes: usize,
    num_genes: usize,
    value_range: Range<F>,
    bounds: Range<F>,
    shapes: Option<Vec<(usize, usize)>>,
    _marker: std::marker::PhantomData<T>,
}

impl<F: Float, T> FloatCodec<F, T> {
    /// Set the bounds of the `FloatGenes` in the [Genotype]. The default bounds
    /// are equal to the min and max values.
    pub fn with_bounds(mut self, range: Range<F>) -> Self {
        self.bounds = range;
        self
    }

    /// Every impl of `Codec` uses the same encode function for the `FloatCodec`, just with a few
    /// different parameters (e.g. `num_chromosomes` and `num_genes`). So, we can just use
    /// the same function for all of them.
    #[inline]
    fn common_encode(&self) -> Genotype<FloatChromosome<F>> {
        if let Some(shapes) = &self.shapes {
            Genotype::from(
                shapes
                    .iter()
                    .map(|(rows, cols)| {
                        FloatChromosome::from((
                            rows * cols,
                            self.value_range.clone(),
                            self.bounds.clone(),
                        ))
                    })
                    .collect::<Vec<FloatChromosome<F>>>(),
            )
        } else {
            Genotype::from(
                (0..self.num_chromosomes)
                    .map(|_| {
                        FloatChromosome::from((
                            self.num_genes,
                            self.value_range.clone(),
                            self.bounds.clone(),
                        ))
                    })
                    .collect::<Vec<FloatChromosome<F>>>(),
            )
        }
    }
}

impl<F: Float> FloatCodec<F, Vec<Vec<Vec<F>>>> {
    pub fn tensor(shapes: Vec<(usize, usize)>, range: Range<F>) -> Self {
        FloatCodec {
            num_chromosomes: shapes.len(),
            num_genes: 0,
            value_range: range.clone(),
            bounds: range,
            shapes: Some(shapes),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F: Float> FloatCodec<F, Vec<Vec<F>>> {
    /// Create a new `FloatCodec` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `FloatGene` will be randomly generated between the min and max values.
    pub fn matrix(rows: usize, cols: usize, range: Range<F>) -> Self {
        FloatCodec {
            num_chromosomes: rows,
            num_genes: cols,
            value_range: range.clone(),
            bounds: range,
            shapes: None,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F: Float> FloatCodec<F, Vec<F>> {
    /// Create a new `FloatCodec` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `FloatGene` will be randomly generated between the min and max values.
    pub fn vector(count: usize, range: Range<F>) -> Self {
        FloatCodec {
            num_chromosomes: 1,
            num_genes: count,
            value_range: range.clone(),
            bounds: range,
            shapes: None,
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
            shapes: None,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Implement the [Codec] for a `FloatCodec` with a `Vec<Vec<Vec<f32>>>` type.
/// Unlike the other impls, this will decode to a 3D tensor of `f32` values.
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// // Create a new FloatCodec with 2 layers:
/// // - First layer: 2 rows and 3 columns
/// // - Second layer: 3 rows and 4 columns
/// let codec = FloatCodec::tensor(vec![(2, 3), (3, 4)], 0.0_f32..1.0_f32);
/// let genotype: Genotype<FloatChromosome<f32>> = codec.encode();
/// let decoded: Vec<Vec<Vec<f32>>> = codec.decode(&genotype);
///
/// assert_eq!(decoded.len(), 2);
/// assert_eq!(decoded[0].len(), 2);
/// assert_eq!(decoded[0][0].len(), 3);
/// assert_eq!(decoded[1].len(), 3);
/// assert_eq!(decoded[1][0].len(), 4);
/// ```
impl<F: Float> Codec<FloatChromosome<F>, Vec<Vec<Vec<F>>>> for FloatCodec<F, Vec<Vec<Vec<F>>>> {
    #[inline]
    fn encode(&self) -> Genotype<FloatChromosome<F>> {
        self.common_encode()
    }

    #[inline]
    fn decode(&self, genotype: &Genotype<FloatChromosome<F>>) -> Vec<Vec<Vec<F>>> {
        if let Some(shapes) = &self.shapes {
            let mut layers = Vec::new();
            for (i, chromosome) in genotype.iter().enumerate() {
                layers.push(
                    chromosome
                        .as_slice()
                        .chunks(shapes[i].1)
                        .map(|chunk| chunk.iter().map(|gene| *gene.allele()).collect::<Vec<F>>())
                        .collect::<Vec<Vec<F>>>(),
                );
            }

            layers
        } else {
            vec![
                genotype
                    .iter()
                    .map(|chromosome| {
                        chromosome
                            .iter()
                            .map(|gene| *gene.allele())
                            .collect::<Vec<F>>()
                    })
                    .collect::<Vec<Vec<F>>>(),
            ]
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
/// let codec = FloatCodec::matrix(3, 4, 0.0_f32..1.0_f32);
/// let genotype: Genotype<FloatChromosome<f32>> = codec.encode();
/// let decoded: Vec<Vec<f32>> = codec.decode(&genotype);
///
/// assert_eq!(decoded.len(), 3);
/// assert_eq!(decoded[0].len(), 4);
/// ```
impl<F: Float> Codec<FloatChromosome<F>, Vec<Vec<F>>> for FloatCodec<F, Vec<Vec<F>>> {
    #[inline]
    fn encode(&self) -> Genotype<FloatChromosome<F>> {
        self.common_encode()
    }

    #[inline]
    fn decode(&self, genotype: &Genotype<FloatChromosome<F>>) -> Vec<Vec<F>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<F>>()
            })
            .collect::<Vec<Vec<F>>>()
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
/// let codec = FloatCodec::vector(3, 0.0_f32..1.0_f32);
/// let genotype: Genotype<FloatChromosome<f32>> = codec.encode();
/// let decoded: Vec<f32> = codec.decode(&genotype);
///
/// assert_eq!(decoded.len(), 3);
/// ```
impl<F: Float> Codec<FloatChromosome<F>, Vec<F>> for FloatCodec<F, Vec<F>> {
    #[inline]
    fn encode(&self) -> Genotype<FloatChromosome<F>> {
        self.common_encode()
    }

    #[inline]
    fn decode(&self, genotype: &Genotype<FloatChromosome<F>>) -> Vec<F> {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<F>>()
            })
            .collect::<Vec<F>>()
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
/// let codec = FloatCodec::scalar(0.0_f32..1.0_f32);
/// let genotype: Genotype<FloatChromosome<f32>> = codec.encode();
/// let decoded: f32 = codec.decode(&genotype);
/// ```
impl<F: Float> Codec<FloatChromosome<F>, F> for FloatCodec<F, F> {
    #[inline]
    fn encode(&self) -> Genotype<FloatChromosome<F>> {
        self.common_encode()
    }

    #[inline]
    fn decode(&self, genotype: &Genotype<FloatChromosome<F>>) -> F {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<F>>()
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
///     FloatChromosome::from((3, 0.0_f32..1.0_f32)),
///     FloatChromosome::from((4, 0.0_f32..1.0_f32)),
/// ];
///
/// let genotype: Genotype<FloatChromosome<f32>> = codec.encode();
/// let decoded: Vec<Vec<f32>> = codec.decode(&genotype);
///
/// assert_eq!(decoded.len(), 2);
/// assert_eq!(decoded[0].len(), 3);
/// assert_eq!(decoded[1].len(), 4);
/// ```
impl<F: Float> Codec<FloatChromosome<F>, Vec<Vec<F>>> for Vec<FloatChromosome<F>> {
    #[inline]
    fn encode(&self) -> Genotype<FloatChromosome<F>> {
        Genotype::from(
            self.iter()
                .map(|chromosome| {
                    chromosome
                        .iter()
                        .map(|gene| gene.new_instance())
                        .collect::<FloatChromosome<F>>()
                })
                .collect::<Vec<FloatChromosome<F>>>(),
        )
    }

    #[inline]
    fn decode(&self, genotype: &Genotype<FloatChromosome<F>>) -> Vec<Vec<F>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<F>>()
            })
            .collect::<Vec<Vec<F>>>()
    }
}

/// Implement the [Codec] trait for a single [FloatChromosome].
/// This is effectively the same as creating a [FloatCodec] vector
///
/// # Example
/// ``` rust
/// use radiate_core::*;
///
/// let codec = FloatChromosome::from((3, 0.0_f32..1.0_f32));
/// let genotype: Genotype<FloatChromosome<f32>> = codec.encode();
/// let decoded: Vec<f32> = codec.decode(&genotype);
///
/// assert_eq!(decoded.len(), 3);
/// ```
impl<F: Float> Codec<FloatChromosome<F>, Vec<F>> for FloatChromosome<F> {
    #[inline]
    fn encode(&self) -> Genotype<FloatChromosome<F>> {
        Genotype::from(
            self.iter()
                .map(|gene| gene.new_instance())
                .collect::<FloatChromosome<F>>(),
        )
    }

    #[inline]
    fn decode(&self, genotype: &Genotype<FloatChromosome<F>>) -> Vec<F> {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<F>>()
            })
            .collect::<Vec<F>>()
    }
}
