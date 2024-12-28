// use crate::Value;

// #[derive(Clone, PartialEq)]
// pub struct TreeCell<T> {
//     pub inner: Option<Value<T>>,
//     pub children: Option<Vec<TreeCell<T>>>,
// }

// impl<T> TreeCell<T> {
//     pub fn new(inner: Value<T>) -> Self {
//         TreeCell {
//             inner: Some(inner),
//             children: None,
//         }
//     }

//     pub fn add_child(&mut self, child: TreeCell<T>) {
//         if self.children.is_none() {
//             self.children = Some(vec![]);
//         }

//         self.children.as_mut().unwrap().push(child);
//     }

//     pub fn children(&self) -> Option<&Vec<TreeCell<T>>> {
//         self.children.as_ref()
//     }

//     pub fn children_mut(&mut self) -> Option<&mut Vec<TreeCell<T>>> {
//         self.children.as_mut()
//     }
// }

// impl<T> AsRef<Value<T>> for TreeCell<T> {
//     fn as_ref(&self) -> &Value<T> {
//         self.inner.as_ref().unwrap()
//     }
// }

// impl<T> From<TreeCell<T>> for Value<T> {
//     fn from(cell: TreeCell<T>) -> Self {
//         cell.inner.unwrap()
//     }
// }

// impl<T> From<Value<T>> for TreeCell<T> {
//     fn from(cell: Value<T>) -> Self {
//         TreeCell {
//             inner: Some(cell),
//             children: None,
//         }
//     }
// }
