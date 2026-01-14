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
        let rank = shape.dimensions();
        if rank == 0 {
            return Self(Arc::from(Vec::<usize>::new()));
        }

        let mut strides = vec![1usize; rank];
        if rank >= 2 {
            for i in (0..rank - 1).rev() {
                let next = shape.dim_at(i + 1).max(1);
                strides[i] = strides[i + 1].saturating_mul(next);
            }
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

    /// Total number of elements implied by this shape.
    /// Uses saturating multiplication to avoid overflow in release builds.
    pub fn size(&self) -> usize {
        self.dims
            .iter()
            .fold(1usize, |acc, &d| acc.saturating_mul(d.max(1)))
    }

    /// Checked total element count. Returns None on overflow.
    pub fn try_size(&self) -> Option<usize> {
        let mut acc = 1usize;
        for &d in self.dims.iter() {
            acc = acc.checked_mul(d.max(1))?;
        }

        Some(acc)
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

    pub fn as_slice(&self) -> &[usize] {
        &self.dims
    }
}

impl AsRef<[usize]> for Shape {
    fn as_ref(&self) -> &[usize] {
        self.as_slice()
    }
}

impl AsRef<[usize]> for Strides {
    fn as_ref(&self) -> &[usize] {
        self.as_slice()
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

/// Compute the row-major flat index for a full N-D index (panics on mismatch/OOB).
#[inline]
pub(crate) fn flat_index_of(shape: &Shape, strides: &Strides, index: &[usize]) -> usize {
    assert_eq!(index.len(), shape.dimensions(), "rank mismatch");
    let mut flat = 0usize;
    for i in 0..index.len() {
        let dim = shape.dim_at(i).max(1);
        let idx = index[i];
        assert!(
            idx < dim,
            "index out of bounds: axis {i} idx={idx} dim={dim}"
        );
        flat = flat.saturating_add(idx.saturating_mul(strides.stride_at(i)));
    }
    flat
}

/// Fallible version of `flat_index_of`.
#[inline]
pub(crate) fn try_flat_index_of(
    shape: &Shape,
    strides: &Strides,
    index: &[usize],
) -> Option<usize> {
    if index.len() != shape.dimensions() {
        return None;
    }
    let mut flat = 0usize;
    for i in 0..index.len() {
        let dim = shape.dim_at(i).max(1);
        let idx = index[i];
        if idx >= dim {
            return None;
        }
        flat = flat.saturating_add(idx.saturating_mul(strides.stride_at(i)));
    }
    Some(flat)
}

/// Compute flat start of a “row” where `axis` varies and other axes fixed by `base` (panics).
#[inline]
pub(crate) fn row_start_flat_of(
    shape: &Shape,
    strides: &Strides,
    base: &[usize],
    axis: usize,
) -> usize {
    assert_eq!(base.len(), shape.rank(), "rank mismatch");
    assert!(axis < shape.rank(), "axis out of bounds");
    let mut flat = 0usize;
    for i in 0..base.len() {
        if i == axis {
            continue;
        }
        let dim = shape.dim_at(i).max(1);
        let idx = base[i];
        assert!(
            idx < dim,
            "index out of bounds: axis {i} idx={idx} dim={dim}"
        );
        flat = flat.saturating_add(idx.saturating_mul(strides.stride_at(i)));
    }
    flat
}
