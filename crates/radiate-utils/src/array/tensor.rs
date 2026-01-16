use crate::array::TensorError;
use crate::{Shape, Strides};
use std::fmt::Debug;

/// Row-major tensor structure. The data is stored in a contiguous vector,
/// and the shape and strides are used to interpret the data.
#[derive(Default)]
pub struct Tensor<T> {
    pub(super) data: Vec<T>,
    pub(super) shape: Shape,
    pub(super) strides: Strides,
}

impl<T> Tensor<T> {
    pub fn new(data: Vec<T>, shape: impl Into<Shape>) -> Self {
        let shape = shape.into();
        let strides = Strides::from(&shape);

        let expected = shape.try_size().unwrap_or_else(|| {
            panic!(
                "Tensor::new: shape size overflow for dims={:?}",
                shape.as_slice()
            )
        });

        assert!(
            data.len() == expected,
            "Tensor::new: data.len()={} does not match shape product {}",
            data.len(),
            expected
        );

        Self {
            data,
            shape,
            strides,
        }
    }

    pub fn try_new(data: Vec<T>, shape: impl Into<Shape>) -> Result<Self, TensorError> {
        let shape = shape.into();
        let strides = Strides::from(&shape);

        let expected = shape.try_size().ok_or_else(|| TensorError::ShapeOverflow {
            dims: shape.as_slice().to_vec(),
        })?;

        if data.len() != expected {
            return Err(TensorError::LenMismatch {
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

    /// The rank (number of dimensions) of the tensor.
    ///
    /// For example, a matrix has rank 2, a vector has rank 1, and a scalar has rank 0.
    ///
    /// ```rust
    /// use radiate_utils::Tensor;
    ///
    /// let two = Tensor::new(vec![1, 2, 3, 4], (2, 2));
    /// let three = Tensor::new(vec![0; 24], (2, 3, 4));
    /// assert_eq!(two.rank(), 2);
    /// assert_eq!(three.rank(), 3);
    /// ```
    #[inline]
    pub fn rank(&self) -> usize {
        self.shape.dimensions()
    }

    /// The dimensions of the tensor. This is essentially a shortcut
    /// for `tensor.shape.as_slice()`. Array of length equal to
    /// the tensor's rank, where each entry is the size of that dimension.
    ///
    /// ```rust
    /// use radiate_utils::Tensor;
    ///
    /// let tensor = Tensor::new(vec![1, 2, 3, 4, 5, 6], (2, 3));
    /// assert_eq!(tensor.dims(), &[2, 3]);
    /// ```
    #[inline]
    pub fn dims(&self) -> &[usize] {
        self.shape.as_slice()
    }

    /// The shape of the tensor. This describes the size of each dimension.
    /// For example, a tensor with shape `[2, 3]` has 2 rows and 3 columns -
    /// essentially a 2x3 matrix.
    ///
    /// ```rust
    /// use radiate_utils::Tensor;
    ///
    /// let tensor = Tensor::new(vec![1, 2, 3, 4, 5, 6], (2, 3));
    /// assert_eq!(tensor.shape().as_slice(), &[2, 3]);
    /// ```
    #[inline]
    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    /// The strides of the tensor. Strides indicate how many elements
    /// to skip in the underlying data vector to move to the next element
    /// along each dimension. For a row-major tensor, the last dimension
    /// has a stride of 1, the second-to-last dimension has a stride equal
    /// to the size of the last dimension, and so on.
    ///
    /// ```rust
    /// use radiate_utils::Tensor;
    ///
    /// let tensor = Tensor::new(vec![1, 2, 3, 4, 5, 6], (2, 3));
    /// assert_eq!(tensor.strides().as_slice(), &[3, 1]);
    /// ```
    #[inline]
    pub fn strides(&self) -> &Strides {
        &self.strides
    }

    /// The underlying data of the tensor as a flat slice.
    /// This data is stored in row-major order.
    ///
    /// ```rust
    /// use radiate_utils::Tensor;
    ///
    /// let tensor = Tensor::new(vec![1, 2, 3, 4, 5, 6], (2, 3));
    /// assert_eq!(tensor.data(), &[1, 2, 3, 4, 5, 6]);
    /// ```
    #[inline]
    pub fn data(&self) -> &[T] {
        &self.data
    }

    /// The underlying data of the tensor as a mutable flat slice.
    /// This data is stored in row-major order.
    ///
    /// ```rust
    /// use radiate_utils::Tensor;
    ///
    /// let mut tensor = Tensor::new(vec![1, 2, 3, 4, 5, 6], (2, 3));
    /// let data_mut = tensor.data_mut();
    /// data_mut[0] = 10;
    /// assert_eq!(tensor.data(), &[10, 2, 3, 4, 5, 6]);
    /// ```
    #[inline]
    pub fn data_mut(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Figure out if the tensor has no elements.
    ///
    /// ```rust
    /// use radiate_utils::Tensor;
    ///
    /// let empty = Tensor::<i32>::new(vec![], (0, 3));
    /// let non_empty = Tensor::new(vec![1, 2, 3], (1, 3));
    /// assert!(empty.is_empty());
    /// assert!(!non_empty.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// --- raw pointers ---
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        self.data.as_ptr()
    }

    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.data.as_mut_ptr()
    }

    #[inline]
    pub fn as_raw_parts(&self) -> (*const T, usize) {
        (self.data.as_ptr(), self.data.len())
    }

    #[inline]
    pub fn as_raw_parts_mut(&mut self) -> (*mut T, usize) {
        (self.data.as_mut_ptr(), self.data.len())
    }

    /// --- iterators ---
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.data.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.data.iter_mut()
    }

    /// Reshape without changing the underlying data.
    /// Panics if the new shape has a different total element count.
    ///
    /// ```rust
    /// use radiate_utils::Tensor;
    ///
    /// let mut tensor = Tensor::new(vec![0, 1, 2, 3, 4, 5], (2, 3));
    /// tensor.reshape((3, 2));
    /// assert_eq!(tensor.shape().as_slice(), &[3, 2]);
    /// assert_eq!(tensor.strides().as_slice(), &[2, 1]); // row-major
    /// assert_eq!(tensor.data(), &[0, 1, 2, 3, 4, 5]);
    /// ```
    #[inline]
    pub fn reshape(&mut self, new_shape: impl Into<Shape>) {
        let new_shape = new_shape.into();
        let expected = new_shape.try_size().unwrap_or_else(|| {
            panic!(
                "Tensor::reshape: shape size overflow for dims={:?}",
                new_shape.as_slice()
            )
        });

        assert!(
            expected == self.data.len(),
            "Tensor::reshape: new shape product {} != data.len() {}",
            expected,
            self.data.len()
        );

        self.shape = new_shape.clone();
        self.strides = Strides::from(&new_shape);
    }
}

impl<T: Clone> Tensor<T> {
    pub fn from_elem(shape: impl Into<Shape>, value: T) -> Self {
        let shape = shape.into();
        let n = shape.try_size().unwrap_or_else(|| {
            panic!(
                "Tensor::from_elem: shape size overflow for dims={:?}",
                shape.as_slice()
            )
        });

        let data = vec![value; n];
        Self::new(data, shape)
    }
}

impl<T: Default + Clone> Tensor<T> {
    pub fn zeros(shape: impl Into<Shape>) -> Self {
        Self::from_elem(shape, T::default())
    }
}

impl<T: Debug> Debug for Tensor<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Tensor(shape={:?}, data=", self.shape.dimensions())?;

        fn fmt_recursive<T: std::fmt::Debug>(
            f: &mut std::fmt::Formatter<'_>,
            data: &[T],
            shape: &[usize],
            strides: &[usize],
            offset: usize,
            depth: usize,
        ) -> std::fmt::Result {
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
    fn test_tensor_basic() {
        let tensor = Tensor::new(vec![1, 2, 3, 4, 5, 6], (2, 3));

        assert_eq!(tensor.rank(), 2);
        assert_eq!(tensor.dims(), &[2, 3]);
        assert_eq!(tensor.shape().as_slice(), &[2, 3]);
        assert_eq!(tensor.strides().as_slice(), &[3, 1]);
        assert_eq!(tensor.data(), &[1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_tensor_from_elem() {
        let tensor = Tensor::from_elem((2, 2), 42);

        assert_eq!(tensor.rank(), 2);
        assert_eq!(tensor.dims(), &[2, 2]);
        assert_eq!(tensor.shape().as_slice(), &[2, 2]);
        assert_eq!(tensor.strides().as_slice(), &[2, 1]);
        assert_eq!(tensor.data(), &[42, 42, 42, 42]);
    }

    #[test]
    fn test_try_new_len_mismatch_err() {
        let err = Tensor::try_new(vec![1, 2, 3], (2, 2)).unwrap_err();
        match err {
            TensorError::LenMismatch { len, expected } => {
                assert_eq!(len, 3);
                assert_eq!(expected, 4);
            }
            other => panic!("expected LenMismatch, got: {:?}", other),
        }
    }

    #[test]
    fn test_reshape_updates_shape_and_strides() {
        let mut t = Tensor::new((0..6).collect::<Vec<i32>>(), (2, 3));

        // reshape to (3, 2)
        t.reshape((3, 2));

        assert_eq!(t.dims(), &[3, 2]);
        assert_eq!(t.strides().as_slice(), &[2, 1]); // row-major
        assert_eq!(t.data(), &[0, 1, 2, 3, 4, 5]);
    }

    #[test]
    #[should_panic]
    fn test_reshape_panics_on_mismatched_size() {
        let mut t = Tensor::new(vec![0; 6], (2, 3));
        t.reshape((2, 2)); // product 4 != 6
    }

    #[test]
    fn test_from_elem_fills_correctly() {
        let t = Tensor::from_elem((2, 3), 7u32);
        assert_eq!(t.data(), &[7, 7, 7, 7, 7, 7]);
        assert_eq!(t.strides().as_slice(), &[3, 1]);
    }

    #[test]
    fn test_zeros_works_for_numeric() {
        let t = Tensor::<i32>::zeros((2, 2, 2));
        assert_eq!(t.data(), &[0; 8]);
        assert_eq!(t.strides().as_slice(), &[4, 2, 1]);
    }

    #[test]
    fn test_as_raw_parts_consistency() {
        let t = Tensor::new(vec![10, 11, 12, 13], (2, 2));
        let (ptr, len) = t.as_raw_parts();
        assert_eq!(len, 4);
        // pointer identity check (safe as long as we don't deref past len)
        assert_eq!(ptr, t.as_ptr());
    }
}
