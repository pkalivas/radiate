use crate::genome::gene::Gene;
use crate::genome::genotype::Genotype;
use crate::{BitChromosome, BitGene, Chromosome};

use super::Codex;

/// A `Codex` for a `Genotype` of `BitGenes`. The `encode` function creates a `Genotype` with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `Vec<Vec<bool>>` from the `Genotype` where the inner `Vec`
/// contains the alleles of the `BitGenes` in the chromosome - the `bool` values.
///
/// # Example
/// ``` rust
/// // In traditional genetic algorithms, a `BitCodex` would be used to create a `Genotype` of `BitGenes`, or a bit string.
/// // This would simply be created by the following:
/// use radiate_core::*;
///
/// // The number of bits (`BitGenes`) in the bit string
/// let length = 10;
///
/// // Create a new `BitCodex` with a single chromosome and `length` genes
/// let codex = BitCodex::matrix(1, length);
///
/// // Create a new `Genotype` of `BitGenes` with a single chromosome and `length` genes
/// let genotype = codex.encode();
///
/// // Decode the `Genotype` to a `Vec<Vec<bool>>`, then get the first chromosome
/// let bit_string: Vec<bool> = codex.decode(&genotype)[0].clone();
/// ```
#[derive(Clone)]
pub struct BitCodex<T = ()> {
    num_chromosomes: usize,
    num_genes: usize,
    _marker: std::marker::PhantomData<T>,
}

impl BitCodex<Vec<Vec<bool>>> {
    pub fn matrix(num_chromosomes: usize, num_genes: usize) -> Self {
        BitCodex {
            num_chromosomes,
            num_genes,
            _marker: std::marker::PhantomData,
        }
    }
}

impl BitCodex<Vec<bool>> {
    pub fn vector(num_genes: usize) -> Self {
        BitCodex {
            num_chromosomes: 1,
            num_genes,
            _marker: std::marker::PhantomData,
        }
    }
}

impl BitCodex<bool> {
    pub fn scalar() -> Self {
        BitCodex {
            num_chromosomes: 1,
            num_genes: 1,
            _marker: std::marker::PhantomData,
        }
    }
}

impl Codex<BitChromosome, Vec<Vec<bool>>> for BitCodex<Vec<Vec<bool>>> {
    fn encode(&self) -> Genotype<BitChromosome> {
        Genotype::new(
            (0..self.num_chromosomes)
                .map(|_| BitChromosome {
                    genes: (0..self.num_genes)
                        .map(|_| BitGene::new())
                        .collect::<Vec<BitGene>>(),
                })
                .collect::<Vec<BitChromosome>>(),
        )
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

impl Codex<BitChromosome, Vec<bool>> for BitCodex<Vec<bool>> {
    fn encode(&self) -> Genotype<BitChromosome> {
        Genotype::new(
            (0..self.num_chromosomes)
                .map(|_| BitChromosome {
                    genes: (0..self.num_genes)
                        .map(|_| BitGene::new())
                        .collect::<Vec<BitGene>>(),
                })
                .collect::<Vec<BitChromosome>>(),
        )
    }

    fn decode(&self, genotype: &Genotype<BitChromosome>) -> Vec<bool> {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<bool>>()
            })
            .collect::<Vec<bool>>()
    }
}

impl Codex<BitChromosome, bool> for BitCodex<bool> {
    fn encode(&self) -> Genotype<BitChromosome> {
        Genotype::new(
            (0..self.num_chromosomes)
                .map(|_| BitChromosome {
                    genes: (0..self.num_genes)
                        .map(|_| BitGene::new())
                        .collect::<Vec<BitGene>>(),
                })
                .collect::<Vec<BitChromosome>>(),
        )
    }

    fn decode(&self, genotype: &Genotype<BitChromosome>) -> bool {
        genotype
            .iter()
            .flat_map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<bool>>()
            })
            .next()
            .unwrap_or(false)
    }
}
