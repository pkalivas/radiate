use crate::{Op, ops::op_names};
use std::{fmt::Display, ops::Range};

impl<T> Op<T> {
    pub fn var(index: usize) -> Self {
        let name = radiate_utils::intern!(format!("X{}", index));
        Op::Var(name, index)
    }

    pub fn vars(range: Range<usize>) -> Vec<Self> {
        range.map(Op::var).collect()
    }

    pub fn named_var(name: impl Into<String>, index: usize) -> Self {
        let name_as_string = name.into();
        let name = radiate_utils::intern!(name_as_string);
        Op::Var(name, index)
    }

    pub fn constant(value: T) -> Self
    where
        T: Display,
    {
        let name = radiate_utils::intern!(format!("{}", value));
        Op::Const(name, value)
    }

    pub fn named_constant(name: &'static str, value: T) -> Self {
        Op::Const(name, value)
    }

    pub fn identity() -> Self
    where
        T: Clone,
    {
        Op::Fn(op_names::IDENTITY, 1.into(), |inputs: &[T]| {
            inputs[0].clone()
        })
    }
}
