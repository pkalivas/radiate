mod any_value;
mod dtype;
mod field;

use std::{any::Any, fmt::Debug};

pub use any_value::AnyValue;
pub use dtype::DataType;
pub use field::Field;

// pub trait RadiateObjectSafe: Any + Debug + Send + Sync {
//     fn type_name(&self) -> &'static str;

//     fn as_any(&self) -> &dyn Any;

//     fn to_boxed(&self) -> Box<dyn RadiateObjectSafe>;

//     fn equal(&self, other: &dyn RadiateObjectSafe) -> bool;
// }

// impl PartialEq for &dyn RadiateObjectSafe {
//     fn eq(&self, other: &Self) -> bool {
//         self.equal(*other)
//     }
// }

// pub trait RadiateObject: Any + Debug + Clone + Send + Sync + Eq {
//     fn type_name() -> &'static str;
// }

// impl<T: RadiateObject> RadiateObjectSafe for T {
//     fn type_name(&self) -> &'static str {
//         T::type_name()
//     }

//     fn as_any(&self) -> &dyn Any {
//         self
//     }

//     fn to_boxed(&self) -> Box<dyn RadiateObjectSafe> {
//         Box::new(self.clone())
//     }

//     fn equal(&self, other: &dyn RadiateObjectSafe) -> bool {
//         let Some(other) = other.as_any().downcast_ref::<T>() else {
//             return false;
//         };
//         self == other
//     }
// }
