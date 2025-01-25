pub mod math;
pub mod mutator;
pub mod operation;
pub mod regression;

pub use operation::*;

pub use math::{get_activation_operations, get_all_operations, get_math_operations};
pub use mutator::OperationMutator;
pub use regression::{DataSet, ErrorFunction, Regression};
