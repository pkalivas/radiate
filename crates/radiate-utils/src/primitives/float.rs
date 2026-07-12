use crate::{DataType, primitives::Primitive};
use num_order::{NumHash, NumOrd};

pub trait Float: Primitive + num_traits::Float + NumHash + NumOrd<Self> {
    const DTYPE: DataType;
    const THREE: Self;
    const FOUR: Self;
    const FIVE: Self;
    const SIX: Self;
    const EPS: Self;
    const NAN: Self;
    const TENTH: Self;

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
    ($t:ty, $dtype:ident, $eps:expr) => {
        impl Primitive for $t {
            const HALF: Self = 0.5;
            const MIN: Self = <$t>::MIN;
            const MAX: Self = <$t>::MAX;
            const ZERO: Self = 0.0;
            const ONE: Self = 1.0;
            const TWO: Self = 2.0;

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

            #[inline]
            fn is_equal(self, rhs: Self) -> bool {
                (self - rhs).abs() <= $eps
            }
        }

        impl Float for $t {
            const DTYPE: DataType = DataType::$dtype;
            const THREE: Self = 3.0;
            const FOUR: Self = 4.0;
            const FIVE: Self = 5.0;
            const SIX: Self = 6.0;
            const NAN: Self = <$t>::NAN;
            const EPS: Self = $eps;
            const TENTH: Self = 0.1;
        }
    };
}

impl_float_scalar!(f32, Float32, 1e-6_f32);
impl_float_scalar!(f64, Float64, 1e-12_f64);
