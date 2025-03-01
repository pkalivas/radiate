pub mod bit;
pub mod chromosome;

pub use bit::{BitChromosome, BitGene};
pub use chromosome::*;
pub mod char;
pub mod float;
pub mod gene;
pub mod int;
pub mod permutation;

use rand::distr::uniform::SampleUniform;
use std::{
    fmt::Debug,
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use crate::{add_impl, arithmetic_impl, div_impl, impl_integer, mul_impl, sub_impl};

pub use char::{CharChromosome, CharGene};
pub use float::{FloatChromosome, FloatGene};
pub use gene::{Gene, NumericGene, Valid};
pub use int::{IntChromosome, IntGene};
pub use permutation::{PermutationChromosome, PermutationGene};

pub trait Integer<T>:
    Copy
    + Clone
    + PartialOrd
    + Debug
    + Add<Output = T>
    + Sub<Output = T>
    + Mul<Output = T>
    + Div<Output = T>
    + SampleUniform
    + Display
    + Default
{
    const MIN: T;
    const MAX: T;

    fn from_i32(value: i32) -> T;
}

// Implement Integer for i8, i16, i32, i64, i128, u8, u16, u32, u64, and u128
impl_integer!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);

// Implement Add, Sub, Mul, and Div for FloatGene and IntGene<i8>,
// IntGene<i16>, IntGene<i32>, IntGene<i64>, IntGene<i128>, IntGene<u8>, IntGene<u16>,
// IntGene<u32>, IntGene<u64>, and IntGene<u128>
arithmetic_impl!(
    FloatGene,
    IntGene<i8>,
    IntGene<i16>,
    IntGene<i32>,
    IntGene<i64>,
    IntGene<i128>,
    IntGene<u8>,
    IntGene<u16>,
    IntGene<u32>,
    IntGene<u64>,
    IntGene<u128>
);
