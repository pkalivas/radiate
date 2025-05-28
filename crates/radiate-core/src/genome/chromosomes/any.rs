use super::{Chromosome, FloatChromosome, Gene, Valid};
use crate::object::*;

#[derive(Clone, Debug, PartialEq)]
pub struct AnyGene<'a> {
    inner: AnyValue<'a>,
}

impl<'a> AnyGene<'a> {
    pub fn new<T: Into<AnyValue<'a>>>(value: T) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl<'a> Gene for AnyGene<'a> {
    type Allele = AnyValue<'a>;

    fn allele(&self) -> &Self::Allele {
        &self.inner
    }

    fn new_instance(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    fn with_allele(&self, allele: &Self::Allele) -> Self {
        Self {
            inner: allele.clone(),
        }
    }
}

impl Valid for AnyGene<'_> {
    fn is_valid(&self) -> bool {
        true
    }
}

impl<'a> From<AnyValue<'a>> for AnyGene<'a> {
    fn from(value: AnyValue<'a>) -> Self {
        Self { inner: value }
    }
}

impl<'a> From<AnyGene<'a>> for AnyValue<'a> {
    fn from(gene: AnyGene<'a>) -> Self {
        gene.inner
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnyChromosome<'a> {
    inner: Vec<AnyGene<'a>>,
}

impl<'a> AnyChromosome<'a> {
    pub fn new<T: Into<AnyValue<'a>>>(alleles: Vec<T>) -> Self {
        Self {
            inner: alleles.into_iter().map(AnyGene::new).collect(),
        }
    }
}

impl<'a> Chromosome for AnyChromosome<'a> {
    type Gene = AnyGene<'a>;
}

impl<'a> Valid for AnyChromosome<'_> {
    fn is_valid(&self) -> bool {
        true
    }
}

impl<'a> AsMut<[AnyGene<'a>]> for AnyChromosome<'a> {
    fn as_mut(&mut self) -> &mut [AnyGene<'a>] {
        &mut self.inner
    }
}

impl<'a> AsRef<[AnyGene<'a>]> for AnyChromosome<'a> {
    fn as_ref(&self) -> &[AnyGene<'a>] {
        &self.inner
    }
}

impl<'a> From<FloatChromosome> for AnyChromosome<'a> {
    fn from(chromosome: FloatChromosome) -> Self {
        let temp = chromosome
            .iter()
            .map(|gene| AnyGene::new(gene.allele().clone()))
            .collect::<Vec<AnyGene<'a>>>();
        todo!()
    }
}
