use std::fmt;
use std::ops::Index;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shape {
    pub dims: Vec<usize>,
}

impl Shape {
    pub fn new(dims: Vec<usize>) -> Self {
        Shape { dims }
    }

    pub fn size(&self) -> usize {
        self.dims.iter().product()
    }

    pub fn dim(&self, index: usize) -> usize {
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
}

impl Into<Shape> for Vec<usize> {
    fn into(self) -> Shape {
        Shape::new(self)
    }
}

impl Into<Shape> for usize {
    fn into(self) -> Shape {
        Shape::new(vec![self])
    }
}

impl Into<Shape> for (usize, usize) {
    fn into(self) -> Shape {
        Shape::new(vec![self.0, self.1])
    }
}

impl Into<Shape> for (usize, usize, usize) {
    fn into(self) -> Shape {
        Shape::new(vec![self.0, self.1, self.2])
    }
}

impl Into<Shape> for (usize, usize, usize, usize) {
    fn into(self) -> Shape {
        Shape::new(vec![self.0, self.1, self.2, self.3])
    }
}

impl Into<Shape> for (usize, usize, usize, usize, usize) {
    fn into(self) -> Shape {
        Shape::new(vec![self.0, self.1, self.2, self.3, self.4])
    }
}

/// Row-major tensor structure. The data is stored in a contiguous vector,
/// and the shape and strides are used to interpret the data.
pub struct Tensor<T> {
    data: Vec<T>,
    shape: Shape,
    strides: Vec<usize>,
}

impl<T> Tensor<T> {
    pub fn new(data: Vec<T>, shape: impl Into<Shape>) -> Self {
        let shape = shape.into();
        let expected_size = shape.size();

        let mut strides = vec![1; shape.rank()];
        for i in (0..shape.rank() - 1).rev() {
            strides[i] = strides[i + 1] * shape.dim(i + 1);
        }

        assert!(
            data.len() == expected_size,
            "Data length {} does not match expected size {} for shape {:?}",
            data.len(),
            expected_size,
            shape
        );

        Self {
            data,
            shape,
            strides,
        }
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }

    pub fn shape(&self) -> &Shape {
        &self.shape
    }

    pub fn strides(&self) -> &[usize] {
        &self.strides
    }
}

impl<T> Index<usize> for Tensor<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> Index<(usize, usize)> for Tensor<T> {
    type Output = T;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        let flat_index = index.0 * self.strides[0] + index.1 * self.strides[1];
        &self.data[flat_index]
    }
}

impl<T> Index<(usize, usize, usize)> for Tensor<T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        let flat_index =
            index.0 * self.strides[0] + index.1 * self.strides[1] + index.2 * self.strides[2];
        &self.data[flat_index]
    }
}

impl<T> Index<(usize, usize, usize, usize)> for Tensor<T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize, usize)) -> &Self::Output {
        let flat_index = index.0 * self.strides[0]
            + index.1 * self.strides[1]
            + index.2 * self.strides[2]
            + index.3 * self.strides[3];
        &self.data[flat_index]
    }
}

impl<T> Index<(usize, usize, usize, usize, usize)> for Tensor<T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize, usize, usize)) -> &Self::Output {
        let flat_index = index.0 * self.strides[0]
            + index.1 * self.strides[1]
            + index.2 * self.strides[2]
            + index.3 * self.strides[3]
            + index.4 * self.strides[4];
        &self.data[flat_index]
    }
}

impl<T: Clone> Clone for Tensor<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            shape: self.shape.clone(),
            strides: self.strides.clone(),
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Tensor<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Tensor(shape={:?}, data=", self.shape.dims)?;

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

        fmt_recursive(f, &self.data, &self.shape.dims, &self.strides, 0, 0)?;

        write!(f, ")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tensor_creation() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let shape = (2, 3);
        let tensor = Tensor::new(data.clone(), shape);
        assert_eq!(tensor.data(), &data);
        assert_eq!(tensor.shape(), &Shape::new(vec![2, 3]));
    }

    #[test]
    fn test_tensor_indexing() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let shape = (2, 3);
        let tensor = Tensor::new(data, shape);
        assert_eq!(tensor[(0, 0)], 1);
        assert_eq!(tensor[(0, 1)], 2);
        assert_eq!(tensor[(1, 2)], 6);
    }
}
