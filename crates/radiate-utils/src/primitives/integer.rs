use crate::{DataType, Primitive};

pub trait Integer: Primitive + num_traits::PrimInt {
    const DTYPE: DataType;

    fn safe_clamp(self, min: Self, max: Self) -> Self {
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}

#[macro_export]
macro_rules! impl_integer {
    ($t:ty, $dtype:ident) => {
        impl Primitive for $t {
            const HALF: Self = 0.5 as Self;
            const MIN: Self = <$t>::MIN;
            const MAX: Self = <$t>::MAX;
            const ZERO: Self = 0;
            const ONE: Self = 1;
            const TWO: Self = 2;

            #[inline]
            fn safe_add(self, rhs: Self) -> Self {
                self.saturating_add(rhs)
            }

            #[inline]
            fn safe_sub(self, rhs: Self) -> Self {
                self.saturating_sub(rhs)
            }

            #[inline]
            fn safe_mul(self, rhs: Self) -> Self {
                self.saturating_mul(rhs)
            }

            #[inline]
            fn safe_div(self, rhs: Self) -> Self {
                if rhs == Self::ZERO {
                    self
                } else {
                    self.saturating_div(rhs)
                }
            }

            #[inline]
            fn safe_mean(self, rhs: Self) -> Self {
                self.safe_add(rhs).safe_div(Self::TWO)
            }

            #[inline]
            fn is_equal(self, rhs: Self) -> bool {
                self == rhs
            }
        }

        impl Integer for $t {
            const DTYPE: DataType = DataType::$dtype;

            fn safe_clamp(self, min: Self, max: Self) -> Self {
                if self < min {
                    min
                } else if self > max {
                    max
                } else {
                    self
                }
            }
        }
    };
}

impl_integer!(i8, Int8);
impl_integer!(i16, Int16);
impl_integer!(i32, Int32);
impl_integer!(i64, Int64);
impl_integer!(i128, Int128);
impl_integer!(u8, UInt8);
impl_integer!(u16, UInt16);
impl_integer!(u32, UInt32);
impl_integer!(u64, UInt64);
impl_integer!(u128, UInt128);
