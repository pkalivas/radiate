pub mod collections;
pub mod ops;
pub mod regression;

pub use collections::*;
pub use ops::{
    get_activation_operations, get_all_operations, get_math_operations, Op, OperationMutator,
};
pub use regression::{Accuracy, AccuracyResult, DataSet, Loss, Regression};

use std::ops::Deref;

/// Arity is a way to describe how many inputs an operation expects.
/// It can be zero, a specific number, or any number.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum Arity {
    Zero,
    Exact(usize),
    Any,
}

impl From<usize> for Arity {
    fn from(value: usize) -> Self {
        match value {
            0 => Arity::Zero,
            n => Arity::Exact(n),
        }
    }
}

impl Deref for Arity {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        match self {
            Arity::Zero => &0,
            Arity::Exact(n) => n,
            Arity::Any => &0,
        }
    }
}
