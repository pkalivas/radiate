use super::Codex;
use crate::engines::genome::gene::Gene;
use crate::engines::genome::genotype::Genotype;
use crate::{Chromosome, FloatChromosome};
use std::ops::Range;

/// A `Codex` for a `Genotype` of `FloatGenes`. The `encode` function creates a `Genotype` with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `Vec<Vec<f32>>` from the `Genotype` where the inner `Vec`
/// contains the alleles of the `FloatGenes` in the chromosome - the `f32` values.
///
/// The lower and upper bounds of the `FloatGenes` can be set with the `with_bounds` function.
/// The default bounds are equal to `min` and `max` values.
#[derive(Clone)]
pub struct FloatCodex {
    num_chromosomes: usize,
    num_genes: usize,
    value_range: Range<f32>,
    bounds: Range<f32>,
}

impl FloatCodex {
    /// Create a new `FloatCodex` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `FloatGene` will be randomly generated between the min and max values.
    pub fn new(num_chromosomes: usize, num_genes: usize, range: Range<f32>) -> Self {
        FloatCodex {
            num_chromosomes,
            num_genes,
            value_range: range.clone(),
            bounds: range,
        }
    }

    /// Set the bounds of the `FloatGenes` in the `Genotype`. The default bounds
    /// are equal to the min and max values.
    pub fn with_bounds(mut self, lower_bound: f32, upper_bound: f32) -> Self {
        self.bounds = lower_bound..upper_bound;
        self
    }
}

impl Codex<FloatChromosome, Vec<Vec<f32>>> for FloatCodex {
    fn encode(&self) -> Genotype<FloatChromosome> {
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

impl Default for FloatCodex {
    fn default() -> Self {
        FloatCodex {
            num_chromosomes: 1,
            num_genes: 1,
            value_range: f32::MIN..f32::MAX,
            bounds: f32::MIN..f32::MAX,
        }
    }
}
