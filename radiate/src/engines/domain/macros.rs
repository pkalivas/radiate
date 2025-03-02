#[macro_export]
macro_rules! impl_integer {
    ($($t:ty),*) => {
        $(
            impl Integer<$t> for $t {
                const MIN: $t = <$t>::MIN;
                const MAX: $t = <$t>::MAX;

                fn from_i32(value: i32) -> $t {
                    value as $t
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! alters {
    ($($struct_instance:expr),* $(,)?) => {
        {
            let mut vec: Vec<Box<dyn Alter<_>>> = Vec::new();
            $(
                vec.push(Box::new($struct_instance.into_alter()));
            )*
            vec
        }
    };
}

#[macro_export]
macro_rules! impl_arithmetic_for_intgene {
    ($gene_type:ty, [$($rhs_type:ty),*]) => {
        use std::ops::{Add, Sub, Mul, Div};

        $(
            impl<T: Integer<T>> Add<$rhs_type> for $gene_type {
                type Output = Self;

                fn add(self, rhs: $rhs_type) -> Self::Output {
                    let new_allele = self.allele + T::from_i32(rhs as i32);
                    self.with_allele(&new_allele)
                }
            }

            impl<T: Integer<T>> Sub<$rhs_type> for $gene_type {
                type Output = Self;

                fn sub(self, rhs: $rhs_type) -> Self::Output {
                    let new_allele = self.allele - T::from_i32(rhs as i32);
                    self.with_allele(&new_allele)
                }
            }

            impl<T: Integer<T>> Mul<$rhs_type> for $gene_type {
                type Output = Self;

                fn mul(self, rhs: $rhs_type) -> Self::Output {
                    let new_allele = self.allele * T::from_i32(rhs as i32);
                    self.with_allele(&new_allele)
                }
            }

            impl<T: Integer<T>> Div<$rhs_type> for $gene_type {
                type Output = Self;

                fn div(self, rhs: $rhs_type) -> Self::Output {
                    let rhs = T::from_i32(rhs as i32);
                    if rhs == T::from_i32(0) {
                        return self;
                    }

                    let new_allele = self.allele / rhs;
                    self.with_allele(&new_allele)

                }
            }
        )*
    };
}
