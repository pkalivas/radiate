use crate::engines::genome::genes::char_gene::CharGene;
use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::genotype::Genotype;
use crate::{CharChromosome, Chromosome};

use super::Codex;

/// A `Codex` for a `Genotype` of `CharGenes`. The `encode` function creates a `Genotype` with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `String` from the `Genotype` where the `String`
/// contains the alleles of the `CharGenes` in the chromosome.
pub struct CharCodex {
    num_chromosomes: usize,
    num_genes: usize,
}

impl CharCodex {
    pub fn new(num_chromosomes: usize, num_genes: usize) -> Self {
        CharCodex {
            num_chromosomes,
            num_genes,
        }
    }
}

impl Codex<CharChromosome, Vec<Vec<char>>> for CharCodex {
    fn encode(&self) -> Genotype<CharChromosome> {
        Genotype {
            chromosomes: (0..self.num_chromosomes)
                .map(|_| {
                    CharChromosome::from_genes(
                        (0..self.num_genes)
                            .map(|_| CharGene::new())
                            .collect::<Vec<CharGene>>(),
                    )
                })
                .collect::<Vec<CharChromosome>>(),
        }
    }

    fn decode(&self, genotype: &Genotype<CharChromosome>) -> Vec<Vec<char>> {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| *gene.allele())
                    .collect::<Vec<char>>()
            })
            .collect::<Vec<Vec<char>>>()
    }
}
