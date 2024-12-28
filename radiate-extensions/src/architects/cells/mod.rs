pub mod expr;
pub mod indexed_cell;
pub mod node_cell;
pub mod tree_cell;

use crate::Direction;
pub use expr::*;
pub use indexed_cell::*;
pub use node_cell::*;
pub use tree_cell::*;

pub trait CellSchema {
    type ValueType;
    fn value(&self) -> &Expr<Self::ValueType>;
    fn id(&self) -> &uuid::Uuid;
    fn enabled(&self) -> bool;
    fn direction(&self) -> Direction;
}
