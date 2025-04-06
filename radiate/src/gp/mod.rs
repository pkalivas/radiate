pub mod collections;
pub mod ops;
pub mod regression;

pub use collections::*;
pub use ops::{Op, OperationMutator, activation_ops, all_ops, math_ops};
pub use regression::{Accuracy, AccuracyResult, DataSet, Loss, Regression};

use std::fmt::Display;
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

impl Display for Arity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Arity::Zero => write!(f, "{:<5}", "Zero"),
            Arity::Exact(n) => write!(f, "{:<5}", n),
            Arity::Any => write!(f, "{:<5}", "Any"),
        }
    }
}
