use crate::Op;
use std::fmt::Display;

impl<T> Op<T> {
    pub fn constant(value: T) -> Self
    where
        T: Display,
    {
        let name = radiate_core::intern!(format!("{}", value));
        Op::Const(name, value)
    }

    pub fn named_constant(name: &'static str, value: T) -> Self {
        Op::Const(name, value)
    }
}
