pub mod collections;
pub mod ops;
pub mod regression;

pub use collections::*;
pub use ops::{
    get_activation_operations, get_all_operations, get_math_operations, Op, OperationMutator,
};
pub use regression::{DataSet, Loss, Regression};
