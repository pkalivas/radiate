use crate::{Direction, Expr};

#[derive(Clone, PartialEq)]
pub struct NodeCell<T> {
    pub value: Expr<T>,
    pub id: uuid::Uuid,
    pub enabled: bool,
    pub direction: Direction,
}

impl<T> NodeCell<T> {
    pub fn new(value: Expr<T>) -> Self {
        NodeCell {
            value,
            id: uuid::Uuid::new_v4(),
            enabled: true,
            direction: Direction::Forward,
        }
    }
}
