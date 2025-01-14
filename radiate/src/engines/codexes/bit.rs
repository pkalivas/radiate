use crate::engines::genome::genes::bit::BitGene;
use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::genotype::Genotype;
use crate::{BitChromosome, Chromosome};

use super::Codex;

/// A `Codex` for a `Genotype` of `BitGenes`. The `encode` function creates a `Genotype` with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `Vec<Vec<bool>>` from the `Genotype` where the inner `Vec`
/// contains the alleles of the `BitGenes` in the chromosome - the `bool` values.
///
/// # Example
/// ``` rust
/// // In traditional genetic algorithms, a `BitCodex` would be used to create a `Genotype` of `BitGenes`, or a bit string.
/// // This would simply be created by the following:
/// use radiate::*;
///
/// // The number of bits (`BitGenes`) in the bit string
/// let length = 10;
///
/// // Create a new `BitCodex` with a single chromosome and `length` genes
/// let codex = BitCodex::new(1, length);
///
/// // Create a new `Genotype` of `BitGenes` with a single chromosome and `length` genes
/// let genotype = codex.encode();
///
/// // Decode the `Genotype` to a `Vec<Vec<bool>>`, then get the first chromosome
/// let bit_string: Vec<bool> = codex.decode(&genotype)[0].clone();
/// ```
pub struct BitCodex {
    num_chromosomes: usize,
    num_genes: usize,
}

impl BitCodex {
    pub fn new(num_chromosomes: usize, num_genes: usize) -> Self {
        BitCodex {
            num_chromosomes,
            num_genes,
        }
    }
}

impl Codex<BitChromosome, Vec<Vec<bool>>> for BitCodex {
    fn encode(&self) -> Genotype<BitChromosome> {
        Genotype {
            chromosomes: (0..self.num_chromosomes)
                .map(|_| BitChromosome {
                    genes: (0..self.num_genes)
                        .map(|_| BitGene::new())
                        .collect::<Vec<BitGene>>(),
                })
                .collect::<Vec<BitChromosome>>(),
        }
    }

    fn decode(&self, genotype: &Genotype<BitChromosome>) -> Vec<Vec<bool>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<bool>>()
            })
            .collect::<Vec<Vec<bool>>>()
    }
}

impl Default for BitCodex {
    fn default() -> Self {
        BitCodex {
            num_chromosomes: 1,
            num_genes: 1,
        }
    }
}
