pub mod bit_gene;
pub mod char_gene;
pub mod float_gene;
pub mod gene;
pub mod generic_gene;
pub mod int_gene;

use rand::distributions::uniform::SampleUniform;
use std::{
    fmt::Debug,
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

pub use bit_gene::*;
pub use char_gene::*;
pub use float_gene::*;
pub use gene::*;
pub use generic_gene::*;
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
{
    const MIN: T;
    const MAX: T;

    fn from_i32(value: i32) -> T;
}

// Implement Integer for i8, i16, i32, i64, and i128
impl Integer<i8> for i8 {
    const MIN: i8 = i8::MIN;
    const MAX: i8 = i8::MAX;

    fn from_i32(value: i32) -> i8 {
        value as i8
    }
}
impl Integer<i16> for i16 {
    const MIN: i16 = i16::MIN;
    const MAX: i16 = i16::MAX;

    fn from_i32(value: i32) -> i16 {
        value as i16
    }
}
impl Integer<i32> for i32 {
    const MIN: i32 = i32::MIN;
    const MAX: i32 = i32::MAX;

    fn from_i32(value: i32) -> i32 {
        value
    }
}
impl Integer<i64> for i64 {
    const MIN: i64 = i64::MIN;
    const MAX: i64 = i64::MAX;

    fn from_i32(value: i32) -> i64 {
        value as i64
    }
}
impl Integer<i128> for i128 {
    const MIN: i128 = i128::MIN;
    const MAX: i128 = i128::MAX;

    fn from_i32(value: i32) -> i128 {
        value as i128
    }
}
