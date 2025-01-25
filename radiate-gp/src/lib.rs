pub mod collections;
pub mod ops;

pub use collections::*;
pub use ops::{
    get_activation_operations, get_all_operations, get_math_operations, DataSet, ErrorFunction, Op,
    OperationMutator, Regression,
};
