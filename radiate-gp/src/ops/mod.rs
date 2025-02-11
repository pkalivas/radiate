pub mod bool;
pub mod math;
pub mod mutator;
pub mod operation;
pub use operation::*;

pub use math::{activation_operations, get_all_operations, math_operations};
pub use mutator::OperationMutator;
