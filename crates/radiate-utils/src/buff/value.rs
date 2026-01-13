use crate::{Shape, Strides};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(PartialEq, Eq, Hash, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Value<T> {
    Scalar(T),
    Array {
        values: Arc<[T]>,
        shape: Shape,
        strides: Strides,
    },
}

impl<T> Value<T> {
    pub fn shape(&self) -> Option<&Shape> {
        match self {
            Value::Array { shape, .. } => Some(shape),
            _ => None,
        }
    }

    pub fn strides(&self) -> Option<&[usize]> {
        match self {
            Value::Scalar(_) => None,
            Value::Array { strides, .. } => Some(&strides.as_slice()),
        }
    }

    pub fn as_scalar(&self) -> Option<&T> {
        match self {
            Value::Scalar(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[T]> {
        match self {
            Value::Array { values, .. } => Some(values),
            _ => None,
        }
    }
}

impl<S, T, F> From<(S, F)> for Value<T>
where
    S: Into<Shape>,
    F: FnMut(usize) -> T,
{
    fn from(value: (S, F)) -> Self {
        let (shape, mut f) = value;
        let dims = shape.into();

        let mut strides = vec![1; dims.rank()];
        for i in (0..dims.rank() - 1).rev() {
            strides[i] = strides[i + 1] * dims.dim_at(i + 1);
        }

        let size = dims.size();
        let mut values = Vec::with_capacity(size);
        for index in 0..size {
            values.push(f(index));
        }

        Value::Array {
            values: Arc::from(values),
            shape: dims.clone(),
            strides: Strides::from(strides),
        }
    }
}

impl<T: Debug> Debug for Value<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Scalar(value) => write!(f, "Scalar({:?})", value),
            Value::Array { shape, strides, .. } => {
                write!(f, "Arr(shape={:?}, strides={:?})", shape, strides)
            }
        }
    }
}
