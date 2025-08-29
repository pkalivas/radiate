use std::{fmt::Debug, sync::Arc};

use crate::AnyValue;
use radiate::{ArithmeticGene, Chromosome, Gene, Valid};

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

impl<'a> ArithmeticGene for AnyGene<'a> {
    fn min(&self) -> &Self::Allele {
        &self.allele
    }

    fn max(&self) -> &Self::Allele {
        &self.allele
    }

    fn mean(&self, other: &Self) -> Self {
        match (self.allele(), other.allele()) {
            (AnyValue::Bool(a), AnyValue::Bool(b)) => AnyGene::new(AnyValue::Bool(*a && *b)),
            (AnyValue::UInt8(a), AnyValue::UInt8(b)) => {
                AnyGene::new(AnyValue::UInt8((*a + *b) / 2))
            }
            (AnyValue::UInt16(a), AnyValue::UInt16(b)) => {
                AnyGene::new(AnyValue::UInt16((*a + *b) / 2))
            }
            (AnyValue::UInt32(a), AnyValue::UInt32(b)) => {
                AnyGene::new(AnyValue::UInt32((*a + *b) / 2))
            }
            (AnyValue::UInt64(a), AnyValue::UInt64(b)) => {
                AnyGene::new(AnyValue::UInt64((*a + *b) / 2))
            }
            (AnyValue::Int8(a), AnyValue::Int8(b)) => AnyGene::new(AnyValue::Int8((*a + *b) / 2)),
            (AnyValue::Int16(a), AnyValue::Int16(b)) => {
                AnyGene::new(AnyValue::Int16((*a + *b) / 2))
            }
            (AnyValue::Int32(a), AnyValue::Int32(b)) => {
                AnyGene::new(AnyValue::Int32((*a + *b) / 2))
            }
            (AnyValue::Int64(a), AnyValue::Int64(b)) => {
                AnyGene::new(AnyValue::Int64((*a + *b) / 2))
            }
            (AnyValue::Int128(a), AnyValue::Int128(b)) => {
                AnyGene::new(AnyValue::Int128((*a + *b) / 2))
            }
            (AnyValue::Float32(a), AnyValue::Float32(b)) => {
                AnyGene::new(AnyValue::Float32((*a + *b) / 2.0))
            }
            (AnyValue::Float64(a), AnyValue::Float64(b)) => {
                AnyGene::new(AnyValue::Float64((*a + *b) / 2.0))
            }
            (AnyValue::Binary(a), AnyValue::Binary(b)) => {
                let m = core::cmp::min(a.len(), b.len());
                let mut out = Vec::with_capacity(m);
                for i in 0..m {
                    let avg = ((a[i] as u16 + b[i] as u16) / 2) as u8;
                    out.push(avg);
                }
                AnyGene::new(AnyValue::Binary(out))
            }
            // Char
            // (AnyValue::Vec(a), AnyValue::Vec(b)) if a.len() == b.len() => {
            //     let v = a
            //         .iter()
            //         .zip(b.iter())
            //         .map(|(x, y)| x.mean(y))
            //         .collect::<Vec<_>>();
            //     AnyGene::new(AnyValue::Vec(Box::new(v)))
            // }
            _ => self.clone(),
        }
    }

    fn from_f32(&self, value: f32) -> Self {
        AnyGene::new(AnyValue::Float32(value))
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
