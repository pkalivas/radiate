use crate::AnyValue;
use radiate::{ArithmeticGene, Chromosome, Gene, Valid, chromosomes::gene::NumericSlotMut};
use std::{collections::HashMap, fmt::Debug, sync::Arc};

type MetaData<'a> = Option<Arc<HashMap<String, String>>>;
type Factory = Arc<dyn Fn() -> AnyValue<'static> + Send + Sync>;

#[derive(Clone)]
pub struct AnyGene<'a> {
    allele: AnyValue<'a>,
    metadata: MetaData<'a>,
    factory: Option<Factory>,
}

impl<'a> AnyGene<'a> {
    pub fn new(allele: AnyValue<'a>) -> Self {
        AnyGene {
            allele,
            factory: None,
            metadata: None,
        }
    }

    pub fn with_factory<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> AnyValue<'static> + Send + Sync + 'static,
    {
        self.factory = Some(Arc::new(factory));
        self
    }

    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(Arc::new(metadata));
        self
    }

    pub fn metadata(&self) -> Option<&HashMap<String, String>> {
        self.metadata.as_ref().map(|m| m.as_ref())
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

    fn allele_mut(&mut self) -> &mut AnyValue<'a> {
        &mut self.allele
    }

    fn new_instance(&self) -> Self {
        if let Some(factory) = &self.factory {
            AnyGene {
                allele: factory(),
                factory: self.factory.clone(),
                metadata: self.metadata.clone(),
            }
        } else {
            self.clone()
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        AnyGene {
            allele: allele.clone(),
            factory: self.factory.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl<'a> ArithmeticGene for AnyGene<'a> {
    fn mean(&self, other: &Self) -> Self {
        if let Some(avg) = super::arithmatic::mean_anyvalue(self.allele(), other.allele()) {
            AnyGene::new(avg)
        } else {
            self.clone()
        }
    }

    fn add(&self, other: Self) -> Self {
        AnyGene {
            allele: self.allele.clone() + other.allele,
            factory: self.factory.clone(),
            metadata: self.metadata.clone(),
        }
    }

    fn sub(&self, other: Self) -> Self {
        AnyGene {
            allele: self.allele.clone() - other.allele,
            factory: self.factory.clone(),
            metadata: self.metadata.clone(),
        }
    }

    fn mul(&self, other: Self) -> Self {
        AnyGene {
            allele: self.allele.clone() * other.allele,
            factory: self.factory.clone(),
            metadata: self.metadata.clone(),
        }
    }

    fn div(&self, other: Self) -> Self {
        AnyGene {
            allele: self.allele.clone() / other.allele,
            factory: self.factory.clone(),
            metadata: self.metadata.clone(),
        }
    }

    fn numeric_slot_mut(&mut self) -> Option<NumericSlotMut<'_>> {
        self.allele.numeric_mut()
    }
}

impl PartialEq for AnyGene<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.allele == other.allele
    }
}

impl Debug for AnyGene<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AnyGene {{ ")?;
        write!(f, "allele: {:?}, ", self.allele)?;
        if let Some(metadata) = &self.metadata {
            write!(f, "metadata: {:?}, ", metadata)?;
        } else {
            write!(f, "metadata: None, ")?;
        }
        write!(f, "}}")
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

impl<'a> From<AnyGene<'a>> for AnyChromosome<'a> {
    fn from(gene: AnyGene<'a>) -> Self {
        AnyChromosome::new(vec![gene])
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
