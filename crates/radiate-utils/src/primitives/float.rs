use crate::{Integer, primitives::Primitive};

pub trait Float: Primitive + num_traits::Float {
    const MIN: Self;
    const MAX: Self;
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    const EPS: Self;

    fn safe_clamp(self, min: Self, max: Self) -> Self {
        if self.is_finite() {
            self.clamp(min, max)
        } else {
            Self::EPS
        }
    }
}

#[macro_export]
macro_rules! impl_float_scalar {
    ($t:ty, $eps:expr) => {
        impl Primitive for $t {
            const HALF: Self = 0.5;

            #[inline]
            fn safe_add(self, rhs: Self) -> Self {
                self + rhs
            }

            #[inline]
            fn safe_sub(self, rhs: Self) -> Self {
                self - rhs
            }

            #[inline]
            fn safe_mul(self, rhs: Self) -> Self {
                (self * rhs)
            }

            #[inline]
            fn safe_div(self, rhs: Self) -> Self {
                if rhs == Self::ZERO { self } else { self / rhs }
            }

            #[inline]
            fn safe_mean(self, rhs: Self) -> Self {
                (self + rhs) * Self::HALF
            }
        }

        impl Float for $t {
            const MIN: Self = <$t>::MIN;
            const MAX: Self = <$t>::MAX;
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;
            const TWO: Self = 2.0;
            const EPS: Self = $eps;
        }
    };
}

impl_float_scalar!(f32, 1e-6_f32);
impl_float_scalar!(f64, 1e-12_f64);
