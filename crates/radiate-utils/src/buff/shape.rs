use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct Strides(Arc<[usize]>);

impl Strides {
    pub fn as_slice(&self) -> &[usize] {
        &self.0
    }

    pub fn stride_at(&self, index: usize) -> usize {
        self.0[index]
    }

    pub fn strides(&self) -> &[usize] {
        &self.0
    }
}

impl From<&[usize]> for Strides {
    fn from(strides: &[usize]) -> Self {
        Self(Arc::from(strides))
    }
}

impl From<Vec<usize>> for Strides {
    fn from(strides: Vec<usize>) -> Self {
        Self(Arc::from(strides))
    }
}

impl From<Shape> for Strides {
    fn from(shape: Shape) -> Self {
        let mut strides = vec![1; shape.dimensions()];
        for i in (0..shape.dimensions() - 1).rev() {
            strides[i] = strides[i + 1] * shape.dim_at(i + 1);
        }

        Self(Arc::from(strides))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct Shape {
    dims: Arc<[usize]>,
}

impl Shape {
    pub fn new(dims: impl Into<Arc<[usize]>>) -> Self {
        let dims = dims.into();
        Self { dims }
    }

    pub fn size(&self) -> usize {
        self.dims.iter().fold(1, |acc, &d| acc * d)
    }

    pub fn dimensions(&self) -> usize {
        self.dims.len()
    }

    pub fn contains_dim(&self, dim: usize) -> bool {
        self.dims.contains(&dim)
    }

    pub fn dim_at(&self, index: usize) -> usize {
        self.dims[index]
    }

    pub fn rank(&self) -> usize {
        self.dims.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dims.is_empty()
    }

    pub fn is_scalar(&self) -> bool {
        self.dims.len() == 1 && self.dims[0] == 1
    }

    pub fn is_vector(&self) -> bool {
        self.dims.len() == 1
    }

    pub fn is_matrix(&self) -> bool {
        self.dims.len() == 2
    }

    pub fn is_tensor(&self) -> bool {
        self.dims.len() > 2
    }

    pub fn is_square(&self) -> bool {
        self.dims.len() == 2 && self.dims[0] == self.dims[1]
    }

    pub fn iter(&self) -> impl Iterator<Item = &usize> {
        self.dims.iter()
    }
}

impl From<&Shape> for Shape {
    fn from(shape: &Shape) -> Self {
        Shape::new(Arc::clone(&shape.dims))
    }
}

impl From<Vec<i32>> for Shape {
    fn from(dims: Vec<i32>) -> Self {
        Shape::new(dims.into_iter().map(|d| d as usize).collect::<Vec<usize>>())
    }
}

impl From<Vec<usize>> for Shape {
    fn from(dims: Vec<usize>) -> Self {
        Shape::new(dims)
    }
}

impl From<usize> for Shape {
    fn from(value: usize) -> Shape {
        Shape::new(vec![value])
    }
}

impl From<(usize, usize)> for Shape {
    fn from(value: (usize, usize)) -> Shape {
        Shape::new(vec![value.0, value.1])
    }
}

impl From<(usize, usize, usize)> for Shape {
    fn from(value: (usize, usize, usize)) -> Shape {
        Shape::new(vec![value.0, value.1, value.2])
    }
}

impl From<(usize, usize, usize, usize)> for Shape {
    fn from(value: (usize, usize, usize, usize)) -> Shape {
        Shape::new(vec![value.0, value.1, value.2, value.3])
    }
}

impl From<(usize, usize, usize, usize, usize)> for Shape {
    fn from(value: (usize, usize, usize, usize, usize)) -> Shape {
        Shape::new(vec![value.0, value.1, value.2, value.3, value.4])
    }
}
