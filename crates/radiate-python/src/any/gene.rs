use std::{fmt::Debug, sync::Arc};

use crate::AnyValue;
use radiate::{Chromosome, Gene, Valid};

#[derive(Clone)]
pub struct AnyGene<'a> {
    allele: AnyValue<'a>,
    factory: Option<Arc<dyn Fn() -> AnyValue<'static> + Send + Sync>>,
}

impl<'a> AnyGene<'a> {
    pub fn new(allele: AnyValue<'a>) -> Self {
        AnyGene {
            allele,
            factory: None,
        }
    }

    pub fn with_factory<F>(self, factory: F) -> Self
    where
        F: Fn() -> AnyValue<'static> + Send + Sync + 'static,
    {
        AnyGene {
            allele: self.allele,
            factory: Some(Arc::new(factory)),
        }
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
        if let Some(factory) = &self.factory {
            AnyGene {
                allele: factory(),
                factory: self.factory.clone(),
            }
        } else {
            self.clone()
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        AnyGene {
            allele: allele.clone(),
            factory: self.factory.clone(),
        }
    }
}

impl PartialEq for AnyGene<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.allele == other.allele
    }
}

impl Debug for AnyGene<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyGene")
            .field("allele", &self.allele)
            .finish()
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

impl<'a> IntoIterator for AnyChromosome<'a> {
    type Item = AnyGene<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.genes.into_iter()
    }
}
