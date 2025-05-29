use super::{ArithmeticGene, Chromosome, Gene, Valid};
use radiate_object::AnyValue;
use std::ops::{Add, Div, Mul, Sub};

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

impl<'a, T> From<T> for AnyGene<'a>
where
    T: Into<AnyValue<'a>>,
{
    fn from(value: T) -> Self {
        Self {
            inner: value.into(),
        }
    }
}

impl Add for AnyGene<'_> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            inner: self.inner + other.inner,
        }
    }
}

impl Sub for AnyGene<'_> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            inner: self.inner - other.inner,
        }
    }
}

impl Mul for AnyGene<'_> {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            inner: self.inner * other.inner,
        }
    }
}

impl Div for AnyGene<'_> {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        Self {
            inner: self.inner / other.inner,
        }
    }
}

impl ArithmeticGene for AnyGene<'_> {
    fn min(&self) -> &Self::Allele {
        &self.inner
    }

    fn max(&self) -> &Self::Allele {
        &self.inner
    }

    fn mean(&self, other: &Self) -> Self {
        Self {
            inner: (self.inner.clone() + other.inner.clone()) / AnyValue::Float32(2.0),
        }
    }

    fn from_f32(&self, value: f32) -> Self {
        Self {
            inner: AnyValue::<'_>::from_numeric(value),
        }
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

    fn genes(&self) -> &[Self::Gene] {
        &self.inner
    }

    fn genes_mut(&mut self) -> &mut [Self::Gene] {
        &mut self.inner
    }
}

impl<'a> Valid for AnyChromosome<'_> {
    fn is_valid(&self) -> bool {
        true
    }
}

impl<'a, T> From<Vec<T>> for AnyChromosome<'a>
where
    T: Into<AnyValue<'a>>,
{
    fn from(genes: Vec<T>) -> Self {
        Self {
            inner: genes.into_iter().map(AnyGene::new).collect(),
        }
    }
}
