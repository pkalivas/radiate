use radiate::{Chromosome, Gene, Valid};

use crate::AnyValue;

#[derive(Clone, Debug, PartialEq)]
pub struct AnyGene<'a> {
    allele: AnyValue<'a>,
}

impl<'a> AnyGene<'a> {
    pub fn new(allele: AnyValue<'a>) -> Self {
        AnyGene { allele }
    }
}

impl Valid for AnyGene<'_> {
    fn is_valid(&self) -> bool {
        true
    }
}

impl<'a> Gene for AnyGene<'a> {
    type Allele = AnyValue<'a>;

    fn allele(&self) -> &Self::Allele {
        &self.allele
    }

    fn new_instance(&self) -> Self {
        self.clone()
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        AnyGene {
            allele: allele.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnyChromosome<'a> {
    genes: Vec<AnyGene<'a>>,
}

impl<'a> AnyChromosome<'a> {
    pub fn new(genes: Vec<AnyGene<'a>>) -> Self {
        AnyChromosome { genes }
    }
}

impl Valid for AnyChromosome<'_> {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|g| g.is_valid())
    }
}

impl<'a> Chromosome for AnyChromosome<'a> {
    type Gene = AnyGene<'a>;

    fn genes(&self) -> &[Self::Gene] {
        &self.genes
    }

    fn genes_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.genes
    }
}

impl<'a> From<Vec<AnyGene<'a>>> for AnyChromosome<'a> {
    fn from(genes: Vec<AnyGene<'a>>) -> Self {
        AnyChromosome::new(genes)
    }
}

impl<'a> FromIterator<AnyGene<'a>> for AnyChromosome<'a> {
    fn from_iter<T: IntoIterator<Item = AnyGene<'a>>>(iter: T) -> Self {
        AnyChromosome::new(iter.into_iter().collect())
    }
}
