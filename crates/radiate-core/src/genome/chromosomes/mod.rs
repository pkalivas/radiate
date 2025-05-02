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

use crate::impl_integer;

pub use char::{CharChromosome, CharGene};
pub use float::{FloatChromosome, FloatGene};
pub use gene::{ArithmeticGene, Gene, Valid};
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

impl_integer!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
