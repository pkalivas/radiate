use crate::AnyValue;
use radiate::{ArithmeticGene, Chromosome, Gene, Valid};
use std::{
    collections::HashMap,
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
    sync::Arc,
};

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

    pub fn numeric_allele_mut(&mut self) -> Option<NumericSlotMut<'_>> {
        self.allele.numeric_mut()
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
}

impl Add for AnyGene<'_> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        AnyGene {
            allele: self.allele + rhs.allele,
            factory: self.factory.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Sub for AnyGene<'_> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        AnyGene {
            allele: self.allele - rhs.allele,
            factory: self.factory.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Mul for AnyGene<'_> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        AnyGene {
            allele: self.allele * rhs.allele,
            factory: self.factory.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

impl Div for AnyGene<'_> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        AnyGene {
            allele: self.allele / rhs.allele,
            factory: self.factory.clone(),
            metadata: self.metadata.clone(),
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

pub enum NumericSlotMut<'a> {
    F32(&'a mut f32),
    F64(&'a mut f64),
    U8(&'a mut u8),
    U16(&'a mut u16),
    U32(&'a mut u32),
    U64(&'a mut u64),
    I8(&'a mut i8),
    I16(&'a mut i16),
    I32(&'a mut i32),
    I64(&'a mut i64),
    I128(&'a mut i128),
}

#[inline(always)]
pub fn apply_numeric_slot_mut(
    slot: NumericSlotMut<'_>,
    mut f_f32: impl FnMut(f32) -> f32,
    mut f_f64: impl FnMut(f64) -> f64,
    mut f_i: impl FnMut(i128, bool) -> i128,
) {
    match slot {
        NumericSlotMut::F32(v) => *v = f_f32(*v),
        NumericSlotMut::F64(v) => *v = f_f64(*v),
        NumericSlotMut::U8(v) => *v = f_i(*v as i128, true).max(0).min(u8::MAX as i128) as u8,
        NumericSlotMut::U16(v) => *v = f_i(*v as i128, true).max(0).min(u16::MAX as i128) as u16,
        NumericSlotMut::U32(v) => *v = f_i(*v as i128, true).max(0).min(u32::MAX as i128) as u32,
        NumericSlotMut::U64(v) => *v = f_i(*v as i128, true).max(0).min(u64::MAX as i128) as u64,
        NumericSlotMut::I8(v) => {
            *v = f_i(*v as i128, false).clamp(i8::MIN as i128, i8::MAX as i128) as i8
        }
        NumericSlotMut::I16(v) => {
            *v = f_i(*v as i128, false).clamp(i16::MIN as i128, i16::MAX as i128) as i16
        }
        NumericSlotMut::I32(v) => {
            *v = f_i(*v as i128, false).clamp(i32::MIN as i128, i32::MAX as i128) as i32
        }
        NumericSlotMut::I64(v) => {
            *v = f_i(*v as i128, false).clamp(i64::MIN as i128, i64::MAX as i128) as i64
        }
        NumericSlotMut::I128(v) => *v = f_i(*v as i128, false),
    }
}

#[inline(always)]
pub fn apply_pair_numeric_slot_mut(
    slot_one: NumericSlotMut<'_>,
    slot_two: NumericSlotMut<'_>,
    mut f_f32: impl FnMut(f32, f32) -> (f32, f32),
    mut f_f64: impl FnMut(f64, f64) -> (f64, f64),
    mut f_i: impl FnMut(i128, i128, bool) -> (i128, i128),
) {
    match (slot_one, slot_two) {
        (NumericSlotMut::F32(v1), NumericSlotMut::F32(v2)) => {
            let (new_v1, new_v2) = f_f32(*v1, *v2);
            *v1 = new_v1;
            *v2 = new_v2;
        }
        (NumericSlotMut::F64(v1), NumericSlotMut::F64(v2)) => {
            let (new_v1, new_v2) = f_f64(*v1, *v2);
            *v1 = new_v1;
            *v2 = new_v2;
        }
        (NumericSlotMut::U8(v1), NumericSlotMut::U8(v2)) => {
            let (new_v1, new_v2) = f_i(*v1 as i128, *v2 as i128, true);
            *v1 = new_v1.max(0).min(u8::MAX as i128) as u8;
            *v2 = new_v2.max(0).min(u8::MAX as i128) as u8;
        }
        (NumericSlotMut::U16(v1), NumericSlotMut::U16(v2)) => {
            let (new_v1, new_v2) = f_i(*v1 as i128, *v2 as i128, true);
            *v1 = new_v1.max(0).min(u16::MAX as i128) as u16;
            *v2 = new_v2.max(0).min(u16::MAX as i128) as u16;
        }
        (NumericSlotMut::U32(v1), NumericSlotMut::U32(v2)) => {
            let (new_v1, new_v2) = f_i(*v1 as i128, *v2 as i128, true);
            *v1 = new_v1.max(0).min(u32::MAX as i128) as u32;
            *v2 = new_v2.max(0).min(u32::MAX as i128) as u32;
        }
        (NumericSlotMut::U64(v1), NumericSlotMut::U64(v2)) => {
            let (new_v1, new_v2) = f_i(*v1 as i128, *v2 as i128, true);
            *v1 = new_v1.max(0).min(u64::MAX as i128) as u64;
            *v2 = new_v2.max(0).min(u64::MAX as i128) as u64;
        }
        (NumericSlotMut::I8(v1), NumericSlotMut::I8(v2)) => {
            let (new_v1, new_v2) = f_i(*v1 as i128, *v2 as i128, false);
            *v1 = new_v1.clamp(i8::MIN as i128, i8::MAX as i128) as i8;
            *v2 = new_v2.clamp(i8::MIN as i128, i8::MAX as i128) as i8;
        }
        (NumericSlotMut::I16(v1), NumericSlotMut::I16(v2)) => {
            let (new_v1, new_v2) = f_i(*v1 as i128, *v2 as i128, false);
            *v1 = new_v1.clamp(i16::MIN as i128, i16::MAX as i128) as i16;
            *v2 = new_v2.clamp(i16::MIN as i128, i16::MAX as i128) as i16;
        }
        (NumericSlotMut::I32(v1), NumericSlotMut::I32(v2)) => {
            let (new_v1, new_v2) = f_i(*v1 as i128, *v2 as i128, false);
            *v1 = new_v1.clamp(i32::MIN as i128, i32::MAX as i128) as i32;
            *v2 = new_v2.clamp(i32::MIN as i128, i32::MAX as i128) as i32;
        }
        (NumericSlotMut::I64(v1), NumericSlotMut::I64(v2)) => {
            let (new_v1, new_v2) = f_i(*v1 as i128, *v2 as i128, false);
            *v1 = new_v1.clamp(i64::MIN as i128, i64::MAX as i128) as i64;
            *v2 = new_v2.clamp(i64::MIN as i128, i64::MAX as i128) as i64;
        }
        (NumericSlotMut::I128(v1), NumericSlotMut::I128(v2)) => {
            let (new_v1, new_v2) = f_i(*v1 as i128, *v2 as i128, false);
            *v1 = new_v1;
            *v2 = new_v2;
        }
        _ => {}
    }
}
