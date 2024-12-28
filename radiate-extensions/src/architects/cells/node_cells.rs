use crate::{Direction, Expr};

#[derive(Clone, PartialEq)]
pub struct NodeCell<T> {
    pub value: Expr<T>,
    pub enabled: bool,
    pub direction: Direction,
}
