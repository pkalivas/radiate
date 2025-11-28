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
pub use gene::{ArithmeticGene, BoundedGene, Gene, Valid};
pub use int::{IntChromosome, IntGene, Integer};
pub use permutation::{PermutationChromosome, PermutationGene};

pub trait NumericAllele {
    fn cast_as_f32(&self) -> Option<f32>;
    fn cast_as_i32(&self) -> Option<i32>;
}

macro_rules! impl_numeric_allele {
    ($($t:ty),*) => {
        $(
            impl NumericAllele for $t {
                fn cast_as_f32(&self) -> Option<f32> {
                    Some(*self as f32)
                }

                fn cast_as_i32(&self) -> Option<i32> {
                    Some(*self as i32)
                }
            }
        )*
    };
}

impl_numeric_allele!(f32, f64, u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
