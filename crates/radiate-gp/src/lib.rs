pub mod collections;
pub mod ops;
pub mod pgm;
pub mod regression;

pub use collections::*;
pub use ops::{Op, OperationMutator, activation_ops, all_ops, math_ops};
pub use pgm::{FactorType, PgmCodec, PgmOp};
pub use regression::{Accuracy, AccuracyResult, DataSet, Loss, Regression};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::Deref;

/// Arity is a way to describe how many inputs an operation expects.
/// It can be zero, a specific number, or any number.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Arity {
    Zero,
    Exact(usize),
    #[default]
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

// #[derive(PartialEq, Eq, Hash, Clone, Debug)]
// #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
// pub enum Domain {
//     Categorical(usize), // number of categories
//     Value,              // any domain
// }

// impl Domain {
//     pub fn is_categorical(&self) -> bool {
//         matches!(self, Domain::Categorical(_))
//     }

//     pub fn is_value(&self) -> bool {
//         matches!(self, Domain::Value)
//     }
// }

// impl Default for Domain {
//     fn default() -> Self {
//         Domain::Value
//     }
// }

// impl Deref for Domain {
//     type Target = usize;

//     fn deref(&self) -> &Self::Target {
//         match self {
//             Domain::Categorical(k) => k,
//             Domain::Value => &0,
//         }
//     }
// }
