use crate::engines::genome::char::CharGene;
use crate::engines::genome::gene::Gene;
use crate::engines::genome::genotype::Genotype;
use crate::{CharChromosome, Chromosome, char};
use std::sync::Arc;

use super::Codex;

/// A `Codex` for a `Genotype` of `CharGenes`. The `encode` function creates a `Genotype` with `num_chromosomes` chromosomes
/// and `num_genes` genes per chromosome. The `decode` function creates a `String` from the `Genotype` where the `String`
/// contains the alleles of the `CharGenes` in the chromosome.
#[derive(Clone)]
pub struct CharCodex {
    num_chromosomes: usize,
    num_genes: usize,
    char_set: Arc<[char]>,
}

impl CharCodex {
    pub fn new(num_chromosomes: usize, num_genes: usize) -> Self {
        CharCodex {
            num_chromosomes,
            num_genes,
            char_set: char::ALPHABET.chars().collect::<Vec<char>>().into(),
        }
    }

    pub fn with_char_set(mut self, char_set: impl Into<Arc<[char]>>) -> Self {
        self.char_set = char_set.into();
        self
    }
}

impl Codex<CharChromosome, Vec<Vec<char>>> for CharCodex {
    fn encode(&self) -> Genotype<CharChromosome> {
        Genotype {
            chromosomes: (0..self.num_chromosomes)
                .map(|_| CharChromosome {
                    genes: (0..self.num_genes)
                        .map(|_| CharGene::new(Arc::clone(&self.char_set)))
                        .collect::<Vec<CharGene>>(),
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
