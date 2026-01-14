use crate::Factory;
use radiate_utils::{Shape, Value};
use std::fmt::Debug;

#[derive(Hash, Clone)]
pub struct Param<T> {
    data: Value<T>,
    supplier: fn(&Value<T>) -> Value<T>,
    modifier: fn(&mut Value<T>),
}

impl<T> Param<T> {
    pub fn new(
        data: impl Into<Value<T>>,
        supplier: fn(&Value<T>) -> Value<T>,
        modifier: fn(&mut Value<T>),
    ) -> Self {
        Param {
            data: data.into(),
            supplier,
            modifier,
        }
    }

    pub fn is_scalar(&self) -> bool {
        matches!(self.data, Value::Scalar(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self.data, Value::Array { .. })
    }

    pub fn data(&self) -> &Value<T> {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut Value<T> {
        &mut self.data
    }

    pub fn shape(&self) -> Option<&Shape> {
        self.data.shape()
    }

    pub fn supplier(&self) -> fn(&Value<T>) -> Value<T> {
        self.supplier
    }

    pub fn modifier(&self) -> fn(&mut Value<T>) {
        self.modifier
    }
}

impl<T> Factory<(), Param<T>> for Param<T> {
    fn new_instance(&self, _: ()) -> Param<T> {
        let data = (self.supplier)(&self.data);
        Param {
            data,
            supplier: self.supplier,
            modifier: self.modifier,
        }
    }
}

impl<T> Factory<Value<T>, Param<T>> for Param<T> {
    fn new_instance(&self, mut val: Value<T>) -> Param<T> {
        (self.modifier)(&mut val);
        Param {
            data: val,
            supplier: self.supplier,
            modifier: self.modifier,
        }
    }
}

impl<T> Default for Param<T>
where
    T: Default,
{
    fn default() -> Self {
        Param {
            data: Value::Scalar(T::default()),
            supplier: |_| Value::Scalar(T::default()),
            modifier: |_| {},
        }
    }
}

impl<T> PartialEq for Param<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T> Debug for Param<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}
