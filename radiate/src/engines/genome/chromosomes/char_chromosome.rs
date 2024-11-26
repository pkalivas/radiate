use crate::{CharGene, Chromosome, Valid};

/// A `Chromosome` that contains `CharGenes`.
#[derive(Clone, PartialEq)]
pub struct CharChromosome {
    pub genes: Vec<CharGene>,
}

impl Chromosome for CharChromosome {
    type GeneType = CharGene;

    fn from_genes(genes: Vec<CharGene>) -> Self {
        CharChromosome { genes }
    }

    fn set_gene(&mut self, index: usize, gene: CharGene) {
        self.genes[index] = gene;
    }

    fn get_genes(&self) -> &[CharGene] {
        &self.genes
    }

    fn get_genes_mut(&mut self) -> &mut [CharGene] {
        &mut self.genes
    }
}

impl Valid for CharChromosome {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|gene| gene.is_valid())
    }
}
