pub mod collections;
pub mod gp;
pub mod ops;

pub use collections::*;
pub use gp::{Accuracy, AccuracyResult, DataSet, Loss, Regression};
pub use ops::{
    get_activation_operations, get_all_operations, get_math_operations, Op, OperationMutator,
    WeightMutator,
};
