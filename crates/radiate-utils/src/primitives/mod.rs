mod float;
mod integer;

use std::fmt::{Debug, Display};

pub use float::Float;
pub use integer::Integer;
use num_traits::{Num, NumCast};
use rand::distr::uniform::SampleUniform;

pub trait Primitive:
    Num + NumCast + Copy + PartialEq + SampleUniform + Debug + Display + Default
{
    const HALF: Self;

    fn safe_add(self, rhs: Self) -> Self;
    fn safe_sub(self, rhs: Self) -> Self;
    fn safe_mul(self, rhs: Self) -> Self;
    fn safe_div(self, rhs: Self) -> Self;
    fn safe_mean(self, rhs: Self) -> Self;

    fn extract<T: NumCast>(self) -> Option<T> {
        T::from(self)
    }
}
