pub mod bit_gene;
pub mod char_gene;
pub mod float_gene;
pub mod gene;
pub mod int_gene;

use rand::{
    distributions::{uniform::SampleUniform, Standard},
    prelude::Distribution,
};
use std::{
    fmt::Debug,
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use crate::{add_impl, div_impl, impl_integer, mul_impl, sub_impl};
pub use bit_gene::*;
pub use char_gene::*;
pub use float_gene::*;
pub use gene::*;
pub use int_gene::*;

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
where
    Standard: Distribution<T>,
{
    const MIN: T;
    const MAX: T;

    fn from_i32(value: i32) -> T;
}

// Implement Integer for i8, i16, i32, i64, and i128
impl_integer!(i8, i16, i32, i64, i128);

// Implement Add, Sub, Mul, and Div for FloatGene and IntGene<i8>, IntGene<i16>, IntGene<i32>, IntGene<i64>, and IntGene<i128>
add_impl!(
    FloatGene,
    IntGene<i8>,
    IntGene<i16>,
    IntGene<i32>,
    IntGene<i64>,
    IntGene<i128>
);
sub_impl!(
    FloatGene,
    IntGene<i8>,
    IntGene<i16>,
    IntGene<i32>,
    IntGene<i64>,
    IntGene<i128>
);
mul_impl!(
    FloatGene,
    IntGene<i8>,
    IntGene<i16>,
    IntGene<i32>,
    IntGene<i64>,
    IntGene<i128>
);
div_impl!(
    FloatGene,
    IntGene<i8>,
    IntGene<i16>,
    IntGene<i32>,
    IntGene<i64>,
    IntGene<i128>
);
