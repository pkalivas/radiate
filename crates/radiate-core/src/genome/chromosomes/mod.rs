pub mod bit;
pub mod char;
pub mod chromosome;
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

pub use bit::{BitChromosome, BitGene};
pub use char::{CharChromosome, CharGene};
pub use chromosome::*;
pub use float::{FloatChromosome, FloatGene};
pub use gene::{ArithmeticGene, BoundedGene, Gene, Valid};
pub use int::{IntChromosome, IntGene};
pub use permutation::{PermutationChromosome, PermutationGene};

pub trait Integer<T>:
    Copy
    + Clone
    + PartialOrd
    + Debug
    + PartialEq
    + Add<Output = T>
    + Sub<Output = T>
    + Mul<Output = T>
    + Div<Output = T>
    + SampleUniform
    + Display
    + Default
where
    T: PartialEq + PartialOrd + Copy + Clone + Debug + Display + Default,
{
    const MIN: T;
    const MAX: T;
    const ZERO: T;
    const ONE: T;
    const TWO: T;

    fn sat_add(self, rhs: T) -> T;
    fn sat_sub(self, rhs: T) -> T;
    fn sat_mul(self, rhs: T) -> T;
    fn sat_div(self, rhs: T) -> T;
    fn clamp(self, min: T, max: T) -> T;
}

impl_integer!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
