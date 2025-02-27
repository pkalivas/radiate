#[macro_export]
macro_rules! add_impl {
    ($($t:ty),*) => {
        $(
            impl Add for $t {
                type Output = $t;

                #[inline]
                fn add(self, other: $t) -> $t {
                    Self {
                        allele: self.allele() + other.allele(),
                        ..self
                    }
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! sub_impl {
    ($($t:ty),*) => {
        $(
            impl Sub for $t {
                type Output = $t;

                #[inline]
                fn sub(self, other: $t) -> $t {
                    Self {
                        allele: self.allele() - other.allele(),
                        ..self
                    }
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! mul_impl {
    ($($t:ty),*) => {
        $(
            impl Mul for $t {
                type Output = $t;

                #[inline]
                fn mul(self, other: $t) -> $t {
                    Self {
                        allele: self.allele() * other.allele(),
                        ..self
                    }
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! div_impl {
    ($($t:ty),*) => {
        $(
            impl Div for $t {
                type Output = $t;

                #[inline]
                fn div(self, other: $t) -> $t {
                    if std::any::TypeId::of::<$t>() == std::any::TypeId::of::<FloatGene>() {
                        if *other.allele() == 0.0 as <$t as Gene>::Allele {
                            return Self {
                                allele: self.allele() / 1.0 as <$t as Gene>::Allele,
                                ..self
                            }
                        }
                    } else if std::any::TypeId::of::<$t>() == std::any::TypeId::of::<IntGene<i8>>() {
                        if *other.allele() == 0 as <$t as Gene>::Allele {
                            return Self {
                                allele: self.allele() / 1 as <$t as Gene>::Allele,
                                ..self
                            }
                        }
                    } else if std::any::TypeId::of::<$t>() == std::any::TypeId::of::<IntGene<i16>>() {
                        if *other.allele() == 0 as <$t as Gene>::Allele {
                            return Self {
                                allele: self.allele() / 1 as <$t as Gene>::Allele,
                                ..self
                            }
                        }
                    } else if std::any::TypeId::of::<$t>() == std::any::TypeId::of::<IntGene<i32>>() {
                        if *other.allele() == 0 as <$t as Gene>::Allele {
                            return Self {
                                allele: self.allele() / 1 as <$t as Gene>::Allele,
                                ..self
                            }
                        }
                    } else if std::any::TypeId::of::<$t>() == std::any::TypeId::of::<IntGene<i64>>() {
                        if *other.allele() == 0 as <$t as Gene>::Allele {
                            return Self {
                                allele: self.allele() / 1 as <$t as Gene>::Allele,
                                ..self
                            }
                        }
                    } else if std::any::TypeId::of::<$t>() == std::any::TypeId::of::<IntGene<i128>>() {
                        if *other.allele() == 0 as <$t as Gene>::Allele {
                            return Self {
                                allele: self.allele() / 1 as <$t as Gene>::Allele,
                                ..self
                            }
                        }
                    } else if std::any::TypeId::of::<$t>() == std::any::TypeId::of::<IntGene<u8>>() {
                        if *other.allele() == 0 as <$t as Gene>::Allele {
                            return Self {
                                allele: self.allele() / 1 as <$t as Gene>::Allele,
                                ..self
                            }
                        }
                    } else if std::any::TypeId::of::<$t>() == std::any::TypeId::of::<IntGene<u16>>() {
                        if *other.allele() == 0 as <$t as Gene>::Allele {
                            return Self {
                                allele: self.allele() / 1 as <$t as Gene>::Allele,
                                ..self
                            }
                        }
                    } else if std::any::TypeId::of::<$t>() == std::any::TypeId::of::<IntGene<u32>>() {
                        if *other.allele() == 0 as <$t as Gene>::Allele {
                            return Self {
                                allele: self.allele() / 1 as <$t as Gene>::Allele,
                                ..self
                            }
                        }
                    } else if std::any::TypeId::of::<$t>() == std::any::TypeId::of::<IntGene<u64>>() {
                        if *other.allele() == 0 as <$t as Gene>::Allele {
                            return Self {
                                allele: self.allele() / 1 as <$t as Gene>::Allele,
                                ..self
                            }
                        }
                    } else if std::any::TypeId::of::<$t>() == std::any::TypeId::of::<IntGene<u128>>() {
                        if *other.allele() == 0 as <$t as Gene>::Allele {
                            return Self {
                                allele: self.allele() / 1 as <$t as Gene>::Allele,
                                ..self
                            }
                        }
                    }

                    Self {
                        allele: self.allele() / *other.allele(),
                        ..self
                    }
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! arithmetic_impl {
    ($($t:ty),*) => {
        $(
            add_impl!($t);
            sub_impl!($t);
            mul_impl!($t);
            div_impl!($t);
        )*
    };
}

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
            let mut vec: Vec<Box<dyn AlterFn<_>>> = Vec::new();
            $(
                vec.push(Box::new($struct_instance.into_alter()));
            )*
            vec
            // let mut vec: Vec<AlterAction<_>> = Vec::new();
            // $(
            //     vec.push($struct_instance.to_alter());
            // )*
            // vec
        }
    };
}
