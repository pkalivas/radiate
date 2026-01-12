use crate::Factory;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(PartialEq, Eq, Hash, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OpData<T> {
    Unit(T),
    Array {
        values: Arc<[T]>,
        strides: Arc<[usize]>,
        dims: Arc<[usize]>,
    },
}

impl<T> OpData<T> {
    pub fn dims(&self) -> Option<&[usize]> {
        match self {
            OpData::Unit(_) => None,
            OpData::Array { dims, .. } => Some(dims),
        }
    }

    pub fn strides(&self) -> Option<&[usize]> {
        match self {
            OpData::Unit(_) => None,
            OpData::Array { strides, .. } => Some(strides),
        }
    }

    pub fn as_scalar(&self) -> Option<&T> {
        match self {
            OpData::Unit(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[T]> {
        match self {
            OpData::Array { values, .. } => Some(values),
            _ => None,
        }
    }
}

impl<T> Factory<(), OpData<T>> for OpData<T>
where
    T: Clone,
{
    fn new_instance(&self, _: ()) -> OpData<T> {
        match self {
            OpData::Unit(value) => OpData::Unit(value.clone()),
            OpData::Array {
                values,
                strides,
                dims,
            } => OpData::Array {
                values: Arc::clone(values),
                strides: Arc::clone(strides),
                dims: Arc::clone(dims),
            },
        }
    }
}

impl<D, T, F> From<(D, F)> for OpData<T>
where
    D: Into<Arc<[usize]>>,
    F: FnMut(usize) -> T,
{
    fn from(value: (D, F)) -> Self {
        let (dims, mut f) = value;
        let dims = dims.into();

        let mut strides = vec![1usize; dims.len()];
        for i in (0..dims.len() - 1).rev() {
            strides[i] = strides[i + 1].saturating_mul(dims[i + 1]);
        }

        let size = dims.iter().product();
        let mut values = Vec::with_capacity(size);
        for index in 0..size {
            values.push(f(index));
        }

        OpData::Array {
            values: Arc::from(values),
            strides: Arc::from(strides),
            dims: Arc::from(dims),
        }
    }
}

#[derive(Hash)]
pub struct OpValue<T> {
    data: OpData<T>,
    supplier: fn(&OpData<T>) -> OpData<T>,
    modifier: fn(&mut OpData<T>),
}

impl<T> OpValue<T> {
    pub fn new(
        data: impl Into<OpData<T>>,
        supplier: fn(&OpData<T>) -> OpData<T>,
        modifier: fn(&mut OpData<T>),
    ) -> Self {
        OpValue {
            data: data.into(),
            supplier,
            modifier,
        }
    }

    pub fn is_scalar(&self) -> bool {
        matches!(self.data, OpData::Unit(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self.data, OpData::Array { .. })
    }

    pub fn data(&self) -> &OpData<T> {
        &self.data
    }

    pub fn dims(&self) -> Option<&[usize]> {
        match &self.data {
            OpData::Unit(_) => None,
            OpData::Array { dims, .. } => Some(dims),
        }
    }

    pub fn supplier(&self) -> fn(&OpData<T>) -> OpData<T> {
        self.supplier
    }

    pub fn modifier(&self) -> fn(&mut OpData<T>) {
        self.modifier
    }
}

impl<T> From<(OpData<T>, &OpValue<T>)> for OpValue<T>
where
    T: Clone,
{
    fn from(value: (OpData<T>, &OpValue<T>)) -> Self {
        let (data, op_value) = value;
        OpValue {
            data: data,
            supplier: op_value.supplier,
            modifier: op_value.modifier,
        }
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

impl<T> Factory<OpData<T>, OpValue<T>> for OpValue<T>
where
    T: Clone,
{
    fn new_instance(&self, mut val: OpData<T>) -> OpValue<T> {
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
            OpData::Unit(value) => OpData::Unit(value.clone()),
            OpData::Array {
                values,
                strides,
                dims,
            } => OpData::Array {
                values: Arc::clone(values),
                strides: Arc::clone(strides),
                dims: Arc::clone(dims),
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
        match &self.data {
            OpData::Unit(value) => write!(f, "Unit({:?})", value),
            OpData::Array {
                values,
                strides,
                dims,
            } => {
                write!(
                    f,
                    "Arr(shape={:?}, dims={:?}, values={:?})",
                    strides, dims, values
                )
            }
        }
    }
}
