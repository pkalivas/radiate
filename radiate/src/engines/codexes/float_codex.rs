use super::Codex;
use crate::engines::genome::chromosome::Chromosome;
use crate::engines::genome::genes::float_gene::FloatGene;
use crate::engines::genome::genes::gene::{BoundGene, Gene};
use crate::engines::genome::genotype::Genotype;
use crate::FloatChromosome;

/// A `Codex` for a `Genotype` of `FloatGenes`. The `encode` function creates a `Genotype` with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `Vec<Vec<f32>>` from the `Genotype` where the inner `Vec`
/// contains the alleles of the `FloatGenes` in the chromosome - the `f32` values.
///
/// The lower and upper bounds of the `FloatGenes` can be set with the `with_bounds` function.
/// The default bounds are `f32::MIN` and `f32::MAX`.
pub struct FloatCodex {
    pub num_chromosomes: usize,
    pub num_genes: usize,
    pub min: f32,
    pub max: f32,
    pub lower_bound: f32,
    pub upper_bound: f32,
}

impl FloatCodex {
    /// Create a new `FloatCodex` with the given number of chromosomes, genes, min, and max values.
    /// The f_32 values for each `FloatGene` will be randomly generated between the min and max values.
    pub fn new(num_chromosomes: usize, num_genes: usize, min: f32, max: f32) -> Self {
        FloatCodex {
            num_chromosomes,
            num_genes,
            min,
            max,
            lower_bound: f32::MIN,
            upper_bound: f32::MAX,
        }
    }

    /// Set the bounds of the `FloatGenes` in the `Genotype`. The default bounds are `f32::MIN` and `f32::MAX`.
    pub fn with_bounds(mut self, lower_bound: f32, upper_bound: f32) -> Self {
        self.lower_bound = lower_bound;
        self.upper_bound = upper_bound;
        self
    }

    /// Create a new `FloatCodex` with a single chromosome and a single gene with the given min and max values.
    /// The default bounds are `f32::MIN` and `f32::MAX`. This is useful for problems where the goal is to find
    /// the best floating point number between the min and max values, like the Rastrigin function.
    pub fn scalar(min: f32, max: f32) -> Self {
        FloatCodex::new(1, 1, min, max)
    }
}

impl Codex<FloatChromosome, Vec<Vec<f32>>> for FloatCodex {
    fn encode(&self) -> Genotype<FloatChromosome> {
        Genotype {
            chromosomes: (0..self.num_chromosomes)
                .map(|_| {
                    FloatChromosome::from_genes(
                        (0..self.num_genes)
                            .map(|_| {
                                FloatGene::new(self.min, self.max)
                                    .with_bounds(self.lower_bound, self.upper_bound)
                            })
                            .collect::<Vec<FloatGene>>(),
                    )
                })
                .collect::<Vec<FloatChromosome>>(),
        }
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
