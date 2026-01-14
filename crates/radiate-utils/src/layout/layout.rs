use crate::LayoutView;
use crate::LayoutViewMut;
use crate::Shape;
use crate::Strides;
use crate::layout::Indices;
use crate::layout::LayoutError;
use std::fmt;

/// Row-major tensor structure. The data is stored in a contiguous vector,
/// and the shape and strides are used to interpret the data.
pub struct Layout<T> {
    pub(super) data: Vec<T>,
    pub(super) shape: Shape,
    pub(super) strides: Strides,
}

impl<T> Layout<T> {
    pub fn new(data: Vec<T>, shape: impl Into<Shape>) -> Self {
        let shape = shape.into();
        let strides = Strides::from(shape.clone());

        // Validate buffer length matches shape.
        let expected = shape.try_size().unwrap_or_else(|| {
            panic!(
                "Layout::new: shape size overflow for dims={:?}",
                shape.as_slice()
            )
        });
        assert!(
            data.len() == expected,
            "Layout::new: data.len()={} does not match shape product {}",
            data.len(),
            expected
        );

        Self {
            data,
            shape,
            strides,
        }
    }

    /// Fallible constructor: returns an error instead of panicking.
    pub fn try_new(data: Vec<T>, shape: impl Into<Shape>) -> Result<Self, LayoutError> {
        let shape = shape.into();
        let strides = Strides::from(shape.clone());
        let expected = shape.try_size().ok_or_else(|| LayoutError::ShapeOverflow {
            dims: shape.as_slice().to_vec(),
        })?;
        if data.len() != expected {
            return Err(LayoutError::LenMismatch {
                len: data.len(),
                expected,
            });
        }
        Ok(Self {
            data,
            shape,
            strides,
        })
    }

    /// Rank (number of dimensions).
    #[inline]
    pub fn rank(&self) -> usize {
        self.shape.rank()
    }

    /// Dimension sizes.
    #[inline]
    pub fn dims(&self) -> &[usize] {
        self.shape.as_slice()
    }

    /// Length of an axis.
    #[inline]
    pub fn axis_len(&self, axis: usize) -> usize {
        self.shape.dim_at(axis).max(1)
    }

    /// Raw pointer to the underlying contiguous buffer.
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.data.as_ptr()
    }

    /// Mutable raw pointer to the underlying contiguous buffer.
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr()
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn strides(&self) -> &Strides {
        &self.strides
    }

    /// Borrow an immutable N-D view.
    pub fn view(&self) -> LayoutView<'_, T> {
        LayoutView {
            data: &self.data,
            shape: &self.shape,
            strides: &self.strides,
        }
    }

    /// Borrow a mutable N-D view.
    pub fn view_mut(&mut self) -> LayoutViewMut<'_, T> {
        LayoutViewMut {
            data: &mut self.data,
            shape: &self.shape,
            strides: &self.strides,
        }
    }

    pub fn len(&self) -> usize {
        self.shape.size()
    }
    /// Fill the entire buffer with a cloned value.
    pub fn fill(&mut self, value: T)
    where
        T: Clone,
    {
        self.data.fill(value);
    }

    /// Copy from a slice with the same length.
    pub fn copy_from_slice(&mut self, src: &[T])
    where
        T: Copy,
    {
        assert_eq!(
            src.len(),
            self.data.len(),
            "Layout::copy_from_slice: src.len()={} != self.len()={}",
            src.len(),
            self.data.len()
        );
        self.data.copy_from_slice(src);
    }

    /// Map into a new layout with the same shape.
    pub fn map<U>(&self, mut f: impl FnMut(&T) -> U) -> Layout<U> {
        let data = self.data.iter().map(|x| f(x)).collect::<Vec<U>>();
        Layout::new(data, self.shape.clone())
    }

    /// Apply a mutation function to all elements in-place.
    pub fn map_in_place(&mut self, mut f: impl FnMut(&mut T)) {
        for x in self.data.iter_mut() {
            f(x);
        }
    }

    /// Zip with another layout of identical shape and map into a new layout.
    pub fn zip_map<U, V>(&self, other: &Layout<U>, mut f: impl FnMut(&T, &U) -> V) -> Layout<V> {
        assert_eq!(
            self.shape.as_slice(),
            other.shape.as_slice(),
            "Layout::zip_map: shape mismatch self={:?} other={:?}",
            self.shape.as_slice(),
            other.shape.as_slice()
        );
        let data = self
            .data
            .iter()
            .zip(other.data.iter())
            .map(|(a, b)| f(a, b))
            .collect::<Vec<V>>();
        Layout::new(data, self.shape.clone())
    }

    /// Iterate over N-D indices (allocates a Vec per item; intended for debugging and generic algorithms).
    pub fn indices(&self) -> Indices {
        Indices::new(self.shape.clone())
    }

    /// Compute the flat index of the start of a row where `axis` varies and all other axes are fixed by `base`.
    /// `base` must have length == rank and will be interpreted with the value on `axis` ignored.
    pub fn row_start_flat(&self, base: &[usize], axis: usize) -> usize {
        super::shape::row_start_flat_of(self.shape(), self.strides(), base, axis)
    }

    /// Return a contiguous slice representing the row where `axis` varies and other axes are fixed by `base`.
    /// This only works when `axis` is the last axis (contiguous in row-major).
    pub fn row_slice(&self, base: &[usize], axis: usize) -> Result<&[T], LayoutError> {
        if axis + 1 != self.rank() {
            return Err(LayoutError::NonContiguousRow { axis });
        }
        if base.len() != self.rank() {
            return Err(LayoutError::RankMismatch {
                got: base.len(),
                expected: self.rank(),
            });
        }
        let start = self.row_start_flat(base, axis);
        let len = self.axis_len(axis);

        Ok(&self.data[start..start + len])
    }

    /// Return a mutable contiguous slice representing the row where `axis` varies and other axes are fixed by `base`.
    /// This only works when `axis` is the last axis (contiguous in row-major).
    pub fn row_slice_mut(&mut self, base: &[usize], axis: usize) -> Result<&mut [T], LayoutError> {
        if axis + 1 != self.rank() {
            return Err(LayoutError::NonContiguousRow { axis });
        }
        if base.len() != self.rank() {
            return Err(LayoutError::RankMismatch {
                got: base.len(),
                expected: self.rank(),
            });
        }
        let start = self.row_start_flat(base, axis);
        let len = self.axis_len(axis);

        Ok(&mut self.data[start..start + len])
    }

    /// Return the flat stride (step) for the given axis.
    #[inline]
    pub fn axis_stride(&self, axis: usize) -> usize {
        self.strides.stride_at(axis)
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Consume and return the underlying contiguous buffer.
    pub fn into_vec(self) -> Vec<T> {
        self.data
    }

    /// Iterator over the underlying contiguous buffer.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    /// Mutable iterator over the underlying contiguous buffer.
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.iter_mut()
    }

    /// Get a value by flat index (bounds-checked by slice).
    pub fn get_flat(&self, idx: usize) -> Option<&T> {
        self.data.get(idx)
    }

    /// Get a mutable value by flat index (bounds-checked by slice).
    pub fn get_flat_mut(&mut self, idx: usize) -> Option<&mut T> {
        self.data.get_mut(idx)
    }

    pub fn flat_index(&self, index: &[usize]) -> usize {
        super::shape::flat_index_of(self.shape(), self.strides(), index)
    }

    pub fn try_flat_index(&self, index: &[usize]) -> Option<usize> {
        super::shape::try_flat_index_of(self.shape(), self.strides(), index)
    }

    /// Generic N-D indexing (panics on rank mismatch / OOB).
    pub fn get_nd(&self, index: &[usize]) -> &T {
        let flat = self.flat_index(index);
        &self.data[flat]
    }

    /// Generic N-D mutable indexing (panics on rank mismatch / OOB).
    pub fn get_nd_mut(&mut self, index: &[usize]) -> &mut T {
        let flat = self.flat_index(index);
        &mut self.data[flat]
    }

    /// Fallible generic N-D indexing.
    pub fn try_get_nd(&self, index: &[usize]) -> Option<&T> {
        let flat = self.try_flat_index(index)?;
        self.data.get(flat)
    }

    /// Fallible generic N-D mutable indexing.
    pub fn try_get_nd_mut(&mut self, index: &[usize]) -> Option<&mut T> {
        let flat = self.try_flat_index(index)?;
        self.data.get_mut(flat)
    }

    /// Iterate over all N-D indices without allocating per item.
    pub fn for_each_indexed(&self, mut f: impl FnMut(&[usize], &T)) {
        let rank = self.rank();
        if rank == 0 {
            return;
        }

        let dims = self.shape.as_slice();
        let mut idx = vec![0usize; rank];

        loop {
            let flat = self.flat_index(&idx);
            f(&idx, &self.data[flat]);

            // increment odometer from last axis
            let mut ax = rank;
            while ax > 0 {
                ax -= 1;
                idx[ax] += 1;
                if idx[ax] < dims[ax].max(1) {
                    break;
                }
                idx[ax] = 0;
                if ax == 0 {
                    return;
                }
            }
        }
    }

    /// Iterate over all N-D indices without allocating per item (mutable).
    pub fn for_each_indexed_mut(&mut self, mut f: impl FnMut(&[usize], &mut T)) {
        let rank = self.rank();
        if rank == 0 {
            return;
        }

        let dims = self.shape.as_slice();
        let mut idx = vec![0usize; rank];

        loop {
            let flat = self.flat_index(&idx);

            // avoid borrow issues by taking one mutable element at a time
            let ptr: *mut T = &mut self.data[flat];
            // SAFETY: flat is in-bounds; we only create one mutable ref at a time
            let cell = unsafe { &mut *ptr };
            f(&idx, cell);

            // increment odometer
            let mut ax = rank;
            while ax > 0 {
                ax -= 1;
                idx[ax] += 1;
                if idx[ax] < dims[ax].max(1) {
                    break;
                }
                idx[ax] = 0;
                if ax == 0 {
                    return;
                }
            }
        }
    }
}

impl<T: Clone> Layout<T> {
    /// Create a new `Layout` filled with `value`.
    pub fn from_elem(shape: impl Into<Shape>, value: T) -> Self {
        let shape = shape.into();
        let n = shape.try_size().unwrap_or_else(|| {
            panic!(
                "Layout::from_elem: shape size overflow for dims={:?}",
                shape.as_slice()
            )
        });

        let data = vec![value; n.max(1)];
        Self::new(data, shape)
    }

    /// Reshape without changing the underlying data.
    /// Panics if the new shape has a different total element count.
    pub fn reshape(&mut self, new_shape: impl Into<Shape>) {
        let new_shape = new_shape.into();
        let expected = new_shape.try_size().unwrap_or_else(|| {
            panic!(
                "Layout::reshape: shape size overflow for dims={:?}",
                new_shape.as_slice()
            )
        });

        assert!(
            expected == self.data.len(),
            "Layout::reshape: new shape product {} != data.len() {}",
            expected,
            self.data.len()
        );

        self.shape = new_shape.clone();
        self.strides = Strides::from(new_shape);
    }
}

impl<T: Default + Clone> Layout<T> {
    pub fn zeros(shape: impl Into<Shape>) -> Self {
        Self::from_elem(shape, T::default())
    }
}

impl<T> Layout<T> {
    pub fn from_shape_fn(shape: impl Into<Shape>, mut f: impl FnMut(usize) -> T) -> Self {
        let shape = shape.into();
        let n = shape.try_size().unwrap_or_else(|| {
            panic!(
                "Layout::from_shape_fn: shape size overflow for dims={:?}",
                shape.as_slice()
            )
        });
        let n = n.max(1);
        let mut data = Vec::with_capacity(n);
        for i in 0..n {
            data.push(f(i));
        }
        Self::new(data, shape)
    }
}

impl<T: Clone> Clone for Layout<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Layout<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Layout(shape={:?}, data=", self.shape.dimensions())?;

        fn fmt_recursive<T: fmt::Debug>(
            f: &mut fmt::Formatter<'_>,
            data: &[T],
            shape: &[usize],
            strides: &[usize],
            offset: usize,
            depth: usize,
        ) -> fmt::Result {
            let indent = " ".repeat(depth);

            if shape.len() == 1 {
                // Vector / leaf
                write!(f, "{}[", indent)?;
                for i in 0..shape[0] {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", data[offset + i * strides[0]])?;
                }
                write!(f, "]")?;
            } else {
                // Higher rank
                write!(f, "{}[", indent)?;
                for i in 0..shape[0] {
                    if i > 0 {
                        writeln!(f, ",")?;
                    } else {
                        writeln!(f)?;
                    }

                    fmt_recursive(
                        f,
                        data,
                        &shape[1..],
                        &strides[1..],
                        offset + i * strides[0],
                        depth + 1,
                    )?;
                }
                writeln!(f)?;
                write!(f, "{}]", indent)?;
            }

            Ok(())
        }

        fmt_recursive(
            f,
            &self.data,
            (0..self.shape.dimensions())
                .map(|i| self.shape.dim_at(i))
                .collect::<Vec<usize>>()
                .as_slice(),
            (0..self.shape.dimensions())
                .map(|i| self.strides.stride_at(i))
                .collect::<Vec<usize>>()
                .as_slice(),
            0,
            0,
        )?;

        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_creation() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let shape = (2, 3);
        let layout = Layout::new(data.clone(), shape);
        assert_eq!(layout.data(), &data);
        assert_eq!(layout.shape(), &Shape::new(vec![2, 3]));
    }

    #[test]
    fn test_layout_indexing() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let shape = (2, 3);
        let layout = Layout::new(data, shape);
        assert_eq!(layout[(0, 0)], 1);
        assert_eq!(layout[(0, 1)], 2);
        assert_eq!(layout[(1, 2)], 6);
    }

    #[test]
    fn test_generic_nd_indexing() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let shape = (2, 3);
        let layout = Layout::new(data, shape);

        // slice-based
        assert_eq!(layout[&[0usize, 0usize][..]], 1);
        assert_eq!(layout[&[1usize, 2usize][..]], 6);

        // const-array based
        assert_eq!(layout[[0usize, 1usize]], 2);
        assert_eq!(layout[[1usize, 0usize]], 4);

        // fallible access
        assert_eq!(layout.try_get_nd(&[9, 9]).is_none(), true);
    }

    #[test]
    fn test_mut_indexing_and_reshape() {
        let mut t = Layout::from_elem((2, 3), 0i32);

        // tuple mut indexing
        t[(1, 2)] = 42;
        assert_eq!(t[(1, 2)], 42);

        // slice mut indexing
        t[&[0usize, 1usize][..]] = 7;
        assert_eq!(t[(0, 1)], 7);

        // const-array mut indexing
        t[[1usize, 0usize]] = 9;
        assert_eq!(t[(1, 0)], 9);

        // reshape to 3x2 (same total elems)
        t.reshape((3, 2));
        assert_eq!(t.shape(), &Shape::new(vec![3, 2]));

        // data preserved; check a couple of flat positions
        assert_eq!(t[0usize], 0);
        assert_eq!(t[5usize], 42);
    }

    #[test]
    fn test_views() {
        let mut t = Layout::from_shape_fn((2, 3), |i| i as i32);

        let v = t.view();
        assert_eq!(v[(0, 0)], 0);
        assert_eq!(v[[1usize, 2usize]], 5);
        assert_eq!(v[&[1usize, 1usize][..]], 4);

        {
            let mut vm = t.view_mut();
            vm[(1, 2)] = 99;
            vm[[0usize, 1usize]] = 77;
        }

        assert_eq!(t[(1, 2)], 99);
        assert_eq!(t[(0, 1)], 77);
    }

    #[test]
    fn test_fill_map_and_zip_map() {
        let mut a = Layout::from_elem((2, 2), 1i32);
        a.fill(3);
        assert_eq!(a[(0, 0)], 3);
        assert_eq!(a[(1, 1)], 3);

        let b = a.map(|x| x * 2);
        assert_eq!(b[(0, 1)], 6);

        let c = a.zip_map(&b, |x, y| x + y);
        assert_eq!(c[(1, 0)], 9); // 3 + 6

        let mut d = Layout::from_elem((2, 2), 0i32);
        d.map_in_place(|x| *x = 5);
        assert_eq!(d[(1, 1)], 5);
    }

    #[test]
    fn test_indices_iter_covers_all() {
        let a = Layout::from_elem((2, 3), 0u8);
        let idxs = a.indices().collect::<Vec<_>>();
        assert_eq!(idxs.len(), 6);
        assert_eq!(idxs[0], vec![0, 0]);
        assert_eq!(idxs[5], vec![1, 2]);
    }

    #[test]
    fn test_row_slice_last_axis() {
        let t = Layout::from_shape_fn((2, 3), |i| i as i32);
        let base = [1usize, 0usize];
        let row = t.row_slice(&base, 1).unwrap();
        assert_eq!(row, &[3, 4, 5]);

        assert!(t.row_slice(&base, 0).is_err());
    }

    #[test]
    fn test_for_each_indexed_sum() {
        let t = Layout::from_shape_fn((2, 3), |i| (i as i32) + 1);
        let mut acc = 0i32;
        t.for_each_indexed(|_idx, v| acc += *v);
        assert_eq!(acc, 21);

        let mut u = Layout::from_elem((2, 3), 0i32);
        u.for_each_indexed_mut(|idx, v| {
            *v = (idx[0] as i32) * 10 + (idx[1] as i32);
        });
        assert_eq!(u[(0, 2)], 2);
        assert_eq!(u[(1, 1)], 11);
    }
}
