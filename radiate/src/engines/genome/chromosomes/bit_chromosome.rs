use crate::{BitGene, Chromosome, Valid};

#[derive(Clone, PartialEq)]
pub struct BitChromosome {
    pub genes: Vec<BitGene>,
}

impl Chromosome for BitChromosome {
    type GeneType = BitGene;

    fn from_genes(genes: Vec<BitGene>) -> Self {
        BitChromosome { genes }
    }

    fn set_gene(&mut self, index: usize, gene: BitGene) {
        self.genes[index] = gene;
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
