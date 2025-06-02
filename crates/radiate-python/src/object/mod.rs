mod any_value;
mod dtype;
mod field;
mod object;

use std::any::Any;

pub use any_value::AnyValue;
pub use dtype::DataType;
pub use field::Field;
pub use object::ObjectValue;

pub trait Object: Any + Clone + PartialEq + Send + Sync {
    fn type_name() -> &'static str;
}

impl<T> Object for T
where
    T: Any + Clone + PartialEq + Send + Sync,
{
    fn type_name() -> &'static str {
        std::any::type_name::<T>()
    }
}

pub trait ObjectSafe: Any + Send + Sync {
    fn type_name(&self) -> &'static str;

    fn as_any(&self) -> &dyn Any;

    fn to_boxed(&self) -> Box<dyn ObjectSafe>;

    fn equals(&self, other: &dyn ObjectSafe) -> bool;
}

impl<T: Object> ObjectSafe for T {
    fn type_name(&self) -> &'static str {
        T::type_name()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn to_boxed(&self) -> Box<dyn ObjectSafe> {
        Box::new(self.clone())
    }

    fn equals(&self, other: &dyn ObjectSafe) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<T>() {
            self == other
        } else {
            false
        }
    }
}

impl PartialEq for dyn ObjectSafe {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
