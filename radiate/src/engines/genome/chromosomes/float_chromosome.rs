use crate::{Chromosome, FloatGene, Valid};

#[derive(Clone, PartialEq)]
pub struct FloatChromosome {
    pub genes: Vec<FloatGene>,
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
