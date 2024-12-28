// use crate::Direction;

// use super::{IndexedValue, Value};

// #[derive(Clone, PartialEq)]
// pub struct GraphCell<T> {
//     pub value: IndexedValue<T>,
//     pub enabled: bool,
//     pub direction: Direction,
// }

// impl<T> GraphCell<T> {
//     pub fn new(value: IndexedValue<T>) -> Self {
//         GraphCell {
//             value,
//             enabled: true,
//             direction: Direction::Forward,
//         }
//     }
// }

// impl<T> From<IndexedValue<T>> for GraphCell<T> {
//     fn from(cell: IndexedValue<T>) -> Self {
//         GraphCell {
//             value: cell,
//             enabled: true,
//             direction: Direction::Forward,
//         }
//     }
// }

// impl<T> From<Value<T>> for GraphCell<T> {
//     fn from(cell: Value<T>) -> Self {
//         GraphCell {
//             value: cell.into(),
//             enabled: true,
//             direction: Direction::Forward,
//         }
//     }
// }

// impl<T> From<GraphCell<T>> for IndexedValue<T> {
//     fn from(cell: GraphCell<T>) -> Self {
//         cell.value
//     }
// }

// impl<T> AsRef<IndexedValue<T>> for GraphCell<T> {
//     fn as_ref(&self) -> &IndexedValue<T> {
//         &self.value
//     }
// }
