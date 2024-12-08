use crate::{CharGene, Chromosome, Valid};

/// A `Chromosome` that contains `CharGenes`.
#[derive(Clone, PartialEq)]
pub struct CharChromosome {
    pub genes: Vec<CharGene>,
}

impl Chromosome for CharChromosome {
    type Gene = CharGene;

    fn from_genes(genes: Vec<CharGene>) -> Self {
        CharChromosome { genes }
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

impl From<&'static str> for CharChromosome {
    fn from(alleles: &'static str) -> Self {
        let genes = alleles.chars().map(CharGene::from).collect();
        CharChromosome { genes }
    }
}
