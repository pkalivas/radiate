use crate::Primitive;

pub trait Integer: Primitive + num_traits::PrimInt {
    const MIN: Self;
    const MAX: Self;
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;

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
    ($($t:ty),*) => {
        $(
            impl Primitive for $t {
                const HALF: Self = 0.5 as Self;

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
            }

            impl Integer for $t {
                const MIN: Self = <$t>::MIN;
                const MAX: Self = <$t>::MAX;
                const ZERO: Self = 0;
                const ONE: Self = 1;
                const TWO: Self = 2;

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
        )*
    };
}

impl_integer!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128);
