use crate::engines::genome::chromosome::Chromosome;
use crate::engines::genome::genes::char_gene::CharGene;
use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::genotype::Genotype;

use super::Codex;

pub struct CharCodex {
    pub num_chromosomes: usize,
    pub num_genes: usize,
}

impl CharCodex {
    pub fn new(num_chromosomes: usize, num_genes: usize) -> Self {
        CharCodex {
            num_chromosomes,
            num_genes,
        }
    }
}

impl Codex<CharGene, char, String> for CharCodex {
    fn encode(&self) -> Genotype<CharGene, char> {
        Genotype {
            chromosomes: (0..self.num_chromosomes)
                .into_iter()
                .map(|_| {
                    Chromosome::from_genes(
                        (0..self.num_genes)
                            .into_iter()
                            .map(|_| CharGene::new())
                            .collect::<Vec<CharGene>>(),
                    )
                })
                .collect::<Vec<Chromosome<CharGene, char>>>(),
        }
    }

    fn decode(&self, genotype: &Genotype<CharGene, char>) -> String {
        genotype
            .iter()
            .map(|chromosome| {
                chromosome
                    .iter()
                    .map(|gene| gene.allele())
                    .collect::<String>()
            })
            .collect::<String>()
    }
}
