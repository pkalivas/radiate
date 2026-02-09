use crate::AnyValue;
use radiate_core::{ArithmeticGene, Chromosome, Gene, Valid};
use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
    sync::Arc,
};

type MetaData = Option<Arc<HashMap<String, String>>>;
type Factory = Option<Arc<dyn Fn() -> AnyValue<'static> + Send + Sync>>;

#[derive(Clone)]
pub struct AnyGene {
    allele: AnyValue<'static>,
    metadata: MetaData,
    factory: Factory,
}

impl AnyGene {
    pub fn new(allele: AnyValue<'static>) -> Self {
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

impl Valid for AnyGene {
    fn is_valid(&self) -> bool {
        true
    }
}

impl Gene for AnyGene {
    type Allele = AnyValue<'static>;

    fn allele(&self) -> &Self::Allele {
        &self.allele
    }

    fn allele_mut(&mut self) -> &mut AnyValue<'static> {
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

impl ArithmeticGene for AnyGene {
    fn mean(&self, other: &Self) -> Self {
        if let Some(avg) = crate::mean_anyvalue(self.allele(), other.allele()) {
            AnyGene {
                allele: avg,
                factory: self.factory.clone(),
                metadata: self.metadata.clone(),
            }
        } else {
            self.clone()
        }
    }
}

impl Add for AnyGene {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        AnyGene {
            allele: self.allele + rhs.allele,
            factory: self.factory.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Sub for AnyGene {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        AnyGene {
            allele: self.allele - rhs.allele,
            factory: self.factory.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Mul for AnyGene {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        AnyGene {
            allele: self.allele * rhs.allele,
            factory: self.factory.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Div for AnyGene {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        AnyGene {
            allele: self.allele / rhs.allele,
            factory: self.factory.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl PartialEq for AnyGene {
    fn eq(&self, other: &Self) -> bool {
        self.allele == other.allele
    }
}

impl Debug for AnyGene {
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
pub struct AnyChromosome {
    genes: Vec<AnyGene>,
}

impl AnyChromosome {
    pub fn new(genes: Vec<AnyGene>) -> Self {
        AnyChromosome { genes }
    }
}

impl Valid for AnyChromosome {
    fn is_valid(&self) -> bool {
        self.genes.iter().all(|g| g.is_valid())
    }
}

impl Chromosome for AnyChromosome {
    type Gene = AnyGene;

    fn as_slice(&self) -> &[Self::Gene] {
        &self.genes
    }

    fn as_mut_slice(&mut self) -> &mut [Self::Gene] {
        &mut self.genes
    }
}

impl From<AnyGene> for AnyChromosome {
    fn from(gene: AnyGene) -> Self {
        AnyChromosome::new(vec![gene])
    }
}

impl From<Vec<AnyGene>> for AnyChromosome {
    fn from(genes: Vec<AnyGene>) -> Self {
        AnyChromosome::new(genes)
    }
}

impl FromIterator<AnyGene> for AnyChromosome {
    fn from_iter<T: IntoIterator<Item = AnyGene>>(iter: T) -> Self {
        AnyChromosome::new(iter.into_iter().collect())
    }
}

impl IntoIterator for AnyChromosome {
    type Item = AnyGene;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.genes.into_iter()
    }
}
