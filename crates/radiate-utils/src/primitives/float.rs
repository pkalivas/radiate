use num_traits::NumCast;
use rand::distr::uniform::SampleUniform;
use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul, Neg, Sub},
};

pub trait Float:
    Copy
    + Clone
    + PartialOrd
    + Debug
    + PartialEq
    + SampleUniform
    + Display
    + Default
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
    + NumCast
{
    const MIN: Self;
    const MAX: Self;
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const EPS: Self;
    const HALF: Self;

    fn is_finite(self) -> bool;
    fn sanitize(self) -> Self;

    fn extract<T: NumCast>(self) -> Option<T> {
        T::from(self)
    }

    fn abs(self) -> Self {
        if self < Self::ZERO { -self } else { self }
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }

    fn min(self, other: Self) -> Self {
        if self < other { self } else { other }
    }
    fn max(self, other: Self) -> Self {
        if self > other { self } else { other }
    }
}

#[macro_export]
macro_rules! impl_float_scalar {
    ($t:ty, $eps:expr) => {
        impl Float for $t {
            const MIN: Self = <$t>::MIN;
            const MAX: Self = <$t>::MAX;
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;
            const TWO: Self = 2.0;
            const HALF: Self = 0.5;
            const EPS: Self = $eps;

            fn is_finite(self) -> bool {
                <$t>::is_finite(self)
            }

            fn sanitize(self) -> Self {
                if self.is_finite() { self } else { Self::EPS }
            }
        }
    };
}

impl_float_scalar!(f32, 1e-6_f32);
impl_float_scalar!(f64, 1e-12_f64);

#[inline]
pub fn safe_div<T: Float>(num: T, den: T) -> T {
    let den = if den.sanitize() == T::ZERO {
        return num.sanitize();
    } else {
        den.sanitize()
    };

    (num.sanitize() / den).sanitize()
}

#[inline]
pub fn safe_add<T: Float>(a: T, b: T) -> T {
    (a.sanitize() + b.sanitize()).sanitize()
}

#[inline]
pub fn safe_sub<T: Float>(a: T, b: T) -> T {
    (a.sanitize() - b.sanitize()).sanitize()
}

#[inline]
pub fn safe_mul<T: Float>(a: T, b: T) -> T {
    (a.sanitize() * b.sanitize()).sanitize()
}
