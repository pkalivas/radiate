use crate::{Chromosome, FloatGene, Valid};
use std::fmt::Debug;
use std::ops::Range;

/// A `Chromosome` that contains `FloatGenes`.
///
/// This can be thought of as a vector of floating point numbers that just has some extra functionality
/// and a name that makes it easier to understand in the context of genetic algorithms.
#[derive(Clone, PartialEq)]
pub struct FloatChromosome {
    pub genes: Vec<FloatGene>,
}

impl FloatChromosome {
    /// Create a new `FloatChromosome` from a `Range<i32>`. This will create a `FloatGene` for each value in the range.
    pub fn from_range(range: Range<i32>) -> Self {
        let mut genes = Vec::new();
        for _ in range.start..range.end {
            genes.push(FloatGene::new(range.start as f32, range.end as f32));
        }

        FloatChromosome { genes }
    }

    pub fn normalize(&mut self) {
        let mut sum = 0.0;
        for gene in &self.genes {
            sum += gene.allele;
        }

        for gene in &mut self.genes {
            gene.allele /= sum;
        }
    }
}

impl Chromosome for FloatChromosome {
    type GeneType = FloatGene;

    fn from_genes(genes: Vec<FloatGene>) -> Self {
        FloatChromosome { genes }
    }

    fn set_gene(&mut self, index: usize, gene: FloatGene) {
        self.genes[index] = gene;
    }

    fn get_genes(&self) -> &[FloatGene] {
        &self.genes
    }

    fn get_genes_mut(&mut self) -> &mut [FloatGene] {
        &mut self.genes
    }
}

impl Valid for FloatChromosome {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}

impl Debug for FloatChromosome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FloatChromosome: [")?;
        for gene in &self.genes {
            write!(f, "{:?}, ", gene)?;
        }
        write!(f, "]")
    }
}
