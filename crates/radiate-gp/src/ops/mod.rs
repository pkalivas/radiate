pub mod bool;
pub mod expr;
pub mod math;
pub mod mutator;
pub mod operation;

pub use expr::Expression;
pub use math::{activation_ops, all_ops, math_ops};
pub use mutator::OperationMutator;
pub use operation::*;
