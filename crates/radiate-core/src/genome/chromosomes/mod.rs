pub mod bit;
pub mod char;
pub mod chromosome;
pub mod float;
pub mod gene;
pub mod int;
pub mod permutation;

pub use bit::{BitChromosome, BitGene};
pub use char::{CharChromosome, CharGene};
pub use chromosome::*;
pub use float::{FloatChromosome, FloatGene};
pub use gene::{ArithmeticGene, BoundedGene, Gene, NumericGene, Valid};
pub use int::{IntChromosome, IntGene};
use num_traits::NumCast;
pub use permutation::{PermutationChromosome, PermutationGene};
use radiate_utils::Primitive;

pub trait NumericAllele: Primitive {
    fn extract<T: NumCast>(self) -> Option<T> {
        T::from(self)
    }
}

macro_rules! impl_numeric_allele {
    ($($t:ty),*) => {
        $(
            impl NumericAllele for $t {}

        )*
    };
}

impl_numeric_allele!(f32, f64, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);

macro_rules! impl_valid {
    ($($t:ty),*) => {
        $(
            impl Valid for $t {
                #[inline]
                fn is_valid(&self) -> bool {
                    true
                }
            }
        )*
    };
}

impl_valid!(
    bool, char, String, isize, usize, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64,
    &str
);
