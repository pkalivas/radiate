use std::fmt;
use std::ops::Index;

use crate::Shape;
use crate::buff::shape::Strides;

/// Row-major tensor structure. The data is stored in a contiguous vector,
/// and the shape and strides are used to interpret the data.
pub struct Tensor<T> {
    data: Vec<T>,
    shape: Shape,
    strides: Strides,
}

impl<T> Tensor<T> {
    pub fn new(data: Vec<T>, shape: impl Into<Shape>) -> Self {
        let shape = shape.into();
        let strides = Strides::from(shape.clone());
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

    pub fn strides(&self) -> &Strides {
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
        let flat_index = index.0 * self.strides.stride_at(0) + index.1 * self.strides.stride_at(1);
        &self.data[flat_index]
    }
}

impl<T> Index<(usize, usize, usize)> for Tensor<T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        let flat_index = index.0 * self.strides.stride_at(0)
            + index.1 * self.strides.stride_at(1)
            + index.2 * self.strides.stride_at(2);
        &self.data[flat_index]
    }
}

impl<T> Index<(usize, usize, usize, usize)> for Tensor<T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize, usize)) -> &Self::Output {
        let flat_index = index.0 * self.strides.stride_at(0)
            + index.1 * self.strides.stride_at(1)
            + index.2 * self.strides.stride_at(2)
            + index.3 * self.strides.stride_at(3);
        &self.data[flat_index]
    }
}

impl<T> Index<(usize, usize, usize, usize, usize)> for Tensor<T> {
    type Output = T;

    fn index(&self, index: (usize, usize, usize, usize, usize)) -> &Self::Output {
        let flat_index = index.0 * self.strides.stride_at(0)
            + index.1 * self.strides.stride_at(1)
            + index.2 * self.strides.stride_at(2)
            + index.3 * self.strides.stride_at(3)
            + index.4 * self.strides.stride_at(4);
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
        writeln!(f, "Tensor(shape={:?}, data=", self.shape.dimensions())?;

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
