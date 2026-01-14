use super::{LayoutError, Shape, Strides};

/// Immutable borrowed N-D view over a row-major contiguous buffer.
#[derive(Clone, Copy, Debug)]
pub struct LayoutView<'a, T> {
    pub(crate) data: &'a [T],
    pub(crate) shape: &'a Shape,
    pub(crate) strides: &'a Strides,
}

/// Mutable borrowed N-D view over a row-major contiguous buffer.
#[derive(Debug)]
pub struct LayoutViewMut<'a, T> {
    pub(crate) data: &'a mut [T],
    pub(crate) shape: &'a Shape,
    pub(crate) strides: &'a Strides,
}

/// Immutable borrowed N-D view that owns its shape/strides (cheap: Shape/Strides use Arc).
#[derive(Clone, Debug)]
pub struct LayoutOwnedView<'a, T> {
    pub(crate) data: &'a [T],
    pub(crate) shape: Shape,
    pub(crate) strides: Strides,
}

/// Mutable borrowed N-D view that owns its shape/strides (cheap: Shape/Strides use Arc).
#[derive(Debug)]
pub struct LayoutOwnedViewMut<'a, T> {
    pub(crate) data: &'a mut [T],
    pub(crate) shape: Shape,
    pub(crate) strides: Strides,
}

impl<'a, T> LayoutView<'a, T> {
    #[inline]
    pub fn data(&self) -> &'a [T] {
        self.data
    }

    #[inline]
    pub fn as_slice(&self) -> &'a [T] {
        self.data
    }

    #[inline]
    pub fn shape(&self) -> &'a Shape {
        self.shape
    }

    #[inline]
    pub fn strides(&self) -> &'a Strides {
        self.strides
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.shape.size()
    }

    /// Reshape this view without changing the underlying data.
    ///
    /// Returns an owned-view wrapper so we don't leak or fight lifetimes.
    pub fn reshape_owned(
        &self,
        new_shape: impl Into<Shape>,
    ) -> Result<LayoutOwnedView<'a, T>, LayoutError> {
        let new_shape = new_shape.into();
        let expected = new_shape
            .try_size()
            .ok_or_else(|| LayoutError::ShapeOverflow {
                dims: new_shape.as_slice().to_vec(),
            })?;
        if expected != self.len() {
            return Err(LayoutError::LenMismatch {
                len: self.len(),
                expected,
            });
        }
        let strides = Strides::from(new_shape.clone());
        Ok(LayoutOwnedView {
            data: self.data,
            shape: new_shape,
            strides,
        })
    }

    #[inline]
    pub fn flat_index(&self, index: &[usize]) -> usize {
        super::shape::flat_index_of(self.shape, self.strides, index)
    }

    #[inline]
    pub fn get_nd(&self, index: &[usize]) -> &T {
        let flat = self.flat_index(index);
        &self.data[flat]
    }
}

impl<'a, T> LayoutViewMut<'a, T> {
    #[inline]
    pub fn data(&self) -> &[T] {
        self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut [T] {
        self.data
    }

    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.data
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.data
    }

    #[inline]
    pub fn shape(&self) -> &Shape {
        self.shape
    }

    #[inline]
    pub fn strides(&self) -> &Strides {
        self.strides
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.shape.size()
    }

    /// Reshape this mutable view without changing the underlying data.
    ///
    /// Returns an owned-view wrapper so we don't leak or fight lifetimes.
    pub fn reshape_owned(
        &mut self,
        new_shape: impl Into<Shape>,
    ) -> Result<LayoutOwnedViewMut<'_, T>, LayoutError> {
        let new_shape = new_shape.into();
        let expected = new_shape
            .try_size()
            .ok_or_else(|| LayoutError::ShapeOverflow {
                dims: new_shape.as_slice().to_vec(),
            })?;
        if expected != self.len() {
            return Err(LayoutError::LenMismatch {
                len: self.len(),
                expected,
            });
        }
        let strides = Strides::from(new_shape.clone());
        Ok(LayoutOwnedViewMut {
            data: self.data,
            shape: new_shape,
            strides,
        })
    }

    #[inline]
    pub fn flat_index(&self, index: &[usize]) -> usize {
        super::shape::flat_index_of(self.shape, self.strides, index)
    }

    #[inline]
    pub fn get_nd(&self, index: &[usize]) -> &T {
        let flat = self.flat_index(index);
        &self.data[flat]
    }

    #[inline]
    pub fn get_nd_mut(&mut self, index: &[usize]) -> &mut T {
        let flat = self.flat_index(index);
        &mut self.data[flat]
    }
}

impl<'a, T> LayoutOwnedView<'a, T> {
    #[inline]
    pub fn data(&self) -> &'a [T] {
        self.data
    }

    #[inline]
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    #[inline]
    pub fn strides(&self) -> &Strides {
        &self.strides
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.shape.size()
    }

    #[inline]
    pub fn flat_index(&self, index: &[usize]) -> usize {
        super::shape::flat_index_of(&self.shape, &self.strides, index)
    }

    #[inline]
    pub fn get_nd(&self, index: &[usize]) -> &T {
        let flat = self.flat_index(index);
        &self.data[flat]
    }
}

impl<'a, T> LayoutOwnedViewMut<'a, T> {
    #[inline]
    pub fn data(&self) -> &[T] {
        self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut [T] {
        self.data
    }

    #[inline]
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    #[inline]
    pub fn strides(&self) -> &Strides {
        &self.strides
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.shape.size()
    }

    #[inline]
    pub fn flat_index(&self, index: &[usize]) -> usize {
        super::shape::flat_index_of(&self.shape, &self.strides, index)
    }

    #[inline]
    pub fn get_nd(&self, index: &[usize]) -> &T {
        let flat = self.flat_index(index);
        &self.data[flat]
    }

    #[inline]
    pub fn get_nd_mut(&mut self, index: &[usize]) -> &mut T {
        let flat = self.flat_index(index);
        &mut self.data[flat]
    }
}
