use crate::engines::genome::chromosome::Chromosome;
use crate::engines::genome::genes::bit_gene::BitGene;
use crate::engines::genome::genes::gene::Gene;
use crate::engines::genome::genotype::Genotype;

use super::Codex;

pub struct BitCodex {
    pub num_chromosomes: usize,
    pub num_genes: usize,
}

impl BitCodex {
    pub fn new(num_chromosomes: usize, num_genes: usize) -> Self {
        BitCodex {
            num_chromosomes,
            num_genes,
        }
    }
}

impl Codex<BitGene, bool, Vec<Vec<bool>>> for BitCodex {
    fn encode(&self) -> Genotype<BitGene, bool> {
        Genotype {
            chromosomes: (0..self.num_chromosomes)
                .into_iter()
                .map(|_| {
                    Chromosome::from_genes(
                        (0..self.num_genes)
                            .into_iter()
                            .map(|_| BitGene::new())
                            .collect::<Vec<BitGene>>(),
                    )
                })
                .collect::<Vec<Chromosome<BitGene, bool>>>(),
        }
    }

    fn decode(&self, genotype: &Genotype<BitGene, bool>) -> Vec<Vec<bool>> {
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
