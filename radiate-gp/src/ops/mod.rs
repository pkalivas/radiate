pub mod bool;
pub mod math;
pub mod mutator;
pub mod operation;
pub use operation::*;

pub use math::{activation_ops, all_ops, math_ops};
pub use mutator::OperationMutator;
