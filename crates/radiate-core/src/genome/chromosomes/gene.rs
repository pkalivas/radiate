/// A [`Valid`] type is a type that can be checked for validity. This is used for checking if a gene
/// or a chromosome is valid. For example, a gene that represents a number between 0 and 1 can be checked
/// for validity by ensuring that the allele is between 0 and 1.
///
/// The `GeneticEngine` will check the validity of the `Chromosome` and `Phenotype` and remove any
/// invalid individuals from the population, replacing them with new individuals at the given generation.
pub trait Valid {
    fn is_valid(&self) -> bool {
        true
    }
}

/// A [`Gene`] is a single unit of information in a `Chromosome`.
/// This is the most basic building block of this entire library.
///
/// Any type that implements this trait can be used as a gene in a chromosome, as such
/// it can be used in any genetic algorithm that uses this library.
///
/// # Example
/// ```
/// use radiate_core::*;
///
/// // A simple gene that represents a point.
/// #[derive(Clone, Debug, PartialEq)]
/// struct PointGene {
///    allele: (f32, f32),
/// }
///
/// // Implement the Gene trait for the PointGene.
/// impl Gene for PointGene {
///     type Allele = (f32, f32);
///
///     fn allele(&self) -> &Self::Allele {
///         &self.allele
///     }
///
///     fn new_instance(&self) -> Self {
///        PointGene { allele: (0.0, 0.0) }
///     }
///
///     fn with_allele(&self, allele: &Self::Allele) -> Self {
///       PointGene { allele: *allele }
///     }
/// }
///
/// // You must also implement the [`Valid`] trait for the gene.
/// // The default implementation of the [`Valid`] trait is to return true.
/// impl Valid for PointGene {
///    fn is_valid(&self) -> bool {
///      let (x, y) = self.allele;
///     // Check if the x and y values are between 0 and 1.
///     x >= 0.0 && x <= 1.0 && y >= 0.0 && y <= 1.0
///   }
/// }
/// ```
pub trait Gene: Clone + Valid {
    type Allele;

    /// Get the `allele` of the [Gene]. This is the value that the [Gene] represents or "expresses".
    fn allele(&self) -> &Self::Allele;

    /// Get a mutable reference to the `allele` of the [Gene].
    fn allele_mut(&mut self) -> &mut Self::Allele;

    /// Create a new instance of the [Gene].
    fn new_instance(&self) -> Self;

    /// Create a new [Gene] with the given `allele`.
    fn with_allele(&self, allele: &Self::Allele) -> Self;
}

pub trait BoundedGene: Gene {
    fn min(&self) -> &Self::Allele;
    fn max(&self) -> &Self::Allele;
    fn bounds(&self) -> (&Self::Allele, &Self::Allele);
}

/// A [Gene] that represents a number. This gene can be used to represent any type of number,
/// including integers, floats, etc. Essentially, any gene that can `Add`, `Sub`, `Mul`, and `Div`
/// can be used as a [ArithmeticGene].
pub trait ArithmeticGene: Gene {
    /// Get the value of the gene as a number.
    fn mean(&self, other: &Self) -> Self;

    fn add(&self, other: Self) -> Self;
    fn sub(&self, other: Self) -> Self;
    fn mul(&self, other: Self) -> Self;
    fn div(&self, other: Self) -> Self;

    fn numeric_slot_mut(&mut self) -> Option<NumericSlotMut<'_>>;
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

/// Primitive alleles that can become a slot.
pub trait NumericAllele {
    fn slot<'a>(&'a mut self) -> NumericSlotMut<'a>;
    fn as_f64(&self) -> f64;
}

macro_rules! impl_numeric_allele {
    ($($t:ty,$name:ident),*) => {
        $(
            impl NumericAllele for $t {
                fn slot<'a>(&'a mut self) -> NumericSlotMut<'a> {
                    NumericSlotMut::$name(self)
                }

                fn as_f64(&self) -> f64 {
                    *self as f64
                }
            }
        )*
    };
}

impl_numeric_allele!(
    f32, F32, f64, F64, u8, U8, u16, U16, u32, U32, u64, U64, i8, I8, i16, I16, i32, I32, i64, I64,
    i128, I128
);

pub trait HasNumericSlot {
    fn numeric_slot_mut(&mut self) -> Option<NumericSlotMut<'_>> {
        None
    }
}

impl<G> HasNumericSlot for G
where
    G: Gene,
    G::Allele: NumericAllele,
{
    fn numeric_slot_mut(&mut self) -> Option<NumericSlotMut<'_>> {
        Some(self.allele_mut().slot())
    }
}

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
