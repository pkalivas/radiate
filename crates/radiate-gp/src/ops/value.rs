use crate::Factory;
use radiate_utils::{Shape, Value};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Hash)]
pub struct OpValue<T> {
    data: Value<T>,
    supplier: fn(&Value<T>) -> Value<T>,
    modifier: fn(&mut Value<T>),
}

impl<T> OpValue<T> {
    pub fn new(
        data: impl Into<Value<T>>,
        supplier: fn(&Value<T>) -> Value<T>,
        modifier: fn(&mut Value<T>),
    ) -> Self {
        OpValue {
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

impl<T> Factory<(), OpValue<T>> for OpValue<T>
where
    T: Clone,
{
    fn new_instance(&self, _: ()) -> OpValue<T> {
        let data = (self.supplier)(&self.data);
        OpValue {
            data,
            supplier: self.supplier,
            modifier: self.modifier,
        }
    }
}

impl<T> Factory<Value<T>, OpValue<T>> for OpValue<T>
where
    T: Clone,
{
    fn new_instance(&self, mut val: Value<T>) -> OpValue<T> {
        (self.modifier)(&mut val);
        OpValue {
            data: val,
            supplier: self.supplier,
            modifier: self.modifier,
        }
    }
}

impl<T> Clone for OpValue<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        let data = match &self.data {
            Value::Scalar(value) => Value::Scalar(value.clone()),
            Value::Array {
                values,
                shape,
                strides,
            } => Value::Array {
                values: Arc::clone(values),
                shape: shape.clone(),
                strides: strides.clone(),
            },
        };

        OpValue {
            data,
            supplier: self.supplier,
            modifier: self.modifier,
        }
    }
}

impl<T> PartialEq for OpValue<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<T> Debug for OpValue<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}
