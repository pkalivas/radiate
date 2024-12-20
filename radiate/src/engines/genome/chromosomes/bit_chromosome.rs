use crate::{BitGene, Chromosome, Valid};

/// A `Chromosome` that contains `BitGenes`.
/// A `BitChromosome` is a collection of `BitGenes` that represent the genetic
/// material of an individual in the population.
///
#[derive(Clone, PartialEq)]
pub struct BitChromosome {
    pub genes: Vec<BitGene>,
}

impl Chromosome for BitChromosome {
    type Gene = BitGene;

    fn from_genes(genes: Vec<BitGene>) -> Self {
        BitChromosome { genes }
    }

    fn get_genes(&self) -> &[BitGene] {
        &self.genes
    }

    fn get_genes_mut(&mut self) -> &mut [BitGene] {
        &mut self.genes
    }
}

impl Valid for BitChromosome {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}

impl From<&[bool]> for BitChromosome {
    fn from(alleles: &[bool]) -> Self {
        let genes = alleles.iter().map(BitGene::from).collect();
        BitChromosome { genes }
    }
}
