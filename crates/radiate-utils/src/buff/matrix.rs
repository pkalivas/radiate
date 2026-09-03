use num_traits::Float;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    fmt::Debug,
    ops::{Index, IndexMut},
};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Matrix<T> {
    data: Vec<T>,
    rows: usize,
    cols: usize,
}

impl<T> Matrix<T> {
    pub fn new(data: impl Into<Vec<T>>) -> Self {
        let data = data.into();
        let rows = data.len();

        Matrix {
            data,
            rows,
            cols: 1,
        }
    }

    pub fn empty() -> Self {
        Matrix {
            data: Vec::new(),
            rows: 0,
            cols: 0,
        }
    }

    pub fn from_rows<I>(rows: I) -> Self
    where
        I: IntoIterator<Item = Vec<T>>,
    {
        let mut iter = rows.into_iter();

        let Some(first) = iter.next() else {
            return Self {
                data: Vec::new(),
                rows: 0,
                cols: 0,
            };
        };

        let cols = first.len();
        let (lower, upper) = iter.size_hint();

        let mut data = Vec::with_capacity(cols.saturating_mul(1 + upper.unwrap_or(lower)));

        data.extend(first);

        let mut row_count = 1;

        for row in iter {
            assert_eq!(row.len(), cols);
            data.extend(row);
            row_count += 1;
        }

        Self {
            data,
            rows: row_count,
            cols,
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn data(&self) -> &[T] {
        &self.data
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.rows = 0;
        self.cols = 0;
    }

    pub fn iter(&self) -> impl Iterator<Item = &[T]> {
        self.data.chunks(self.cols)
    }

    pub fn row(&self, row: usize) -> &[T] {
        let start = row * self.cols;
        let end = start + self.cols;

        &self.data[start..end]
    }

    pub fn row_mut(&mut self, row: usize) -> &mut [T] {
        let start = row * self.cols;
        let end = start + self.cols;

        &mut self.data[start..end]
    }

    pub fn append_row(&mut self, row_data: Vec<T>) {
        if self.is_empty() {
            self.cols = row_data.len();
        } else {
            assert!(
                row_data.len() == self.cols,
                "Row length must match the number of columns"
            );
        }

        self.data.extend(row_data);
        self.rows += 1;
    }

    pub fn reshape(mut self, new_rows: usize, new_cols: usize) -> Self {
        assert!(
            new_rows * new_cols == self.data.len(),
            "New dimensions must match the total number of elements"
        );
        self.rows = new_rows;
        self.cols = new_cols;
        self
    }

    pub fn reshape_in_place(&mut self, new_rows: usize, new_cols: usize) {
        assert!(
            new_rows * new_cols == self.data.len(),
            "New dimensions must match the total number of elements"
        );
        self.rows = new_rows;
        self.cols = new_cols;
    }
}

impl<T: Clone> Matrix<T> {
    pub fn append_column(&mut self, col_data: Vec<T>) {
        debug_assert!(col_data.len() == self.rows);
        let mut new_data = Vec::with_capacity((self.rows + 1) * self.cols);
        for (row, col) in self.data.chunks(self.cols).zip(col_data.into_iter()) {
            new_data.extend_from_slice(row);
            new_data.push(col);
        }
        self.data = new_data;
        self.cols += 1;
    }

    pub fn reshape_and_fill(&mut self, rows: usize, cols: usize, default_value: T) {
        let new_size = rows * cols;
        self.data.clear();
        self.data.resize(new_size, default_value);
        self.rows = rows;
        self.cols = cols;
    }

    pub fn sort_by_indices(&self, indices: &[usize]) -> Self {
        debug_assert!(
            indices.iter().all(|&i| i < self.rows),
            "row index out of bounds"
        );

        let mut data = Vec::with_capacity(indices.len() * self.cols);

        for &row in indices {
            let start = row * self.cols;
            let end = start + self.cols;
            data.extend_from_slice(&self.data[start..end]);
        }

        Matrix {
            data,
            rows: indices.len(),
            cols: self.cols,
        }
    }

    pub fn split_at_row(&self, row: usize) -> (Self, Self) {
        debug_assert!(row <= self.rows);

        let first_part_data = self.data[0..row * self.cols].to_vec();
        let second_part_data = self.data[row * self.cols..].to_vec();

        let first_part = Matrix {
            data: first_part_data,
            rows: row,
            cols: self.cols,
        };

        let second_part = Matrix {
            data: second_part_data,
            rows: self.rows - row,
            cols: self.cols,
        };

        (first_part, second_part)
    }

    pub fn fill(&mut self, value: T) {
        self.data.fill(value);
    }

    pub fn transpose(&self) -> Self {
        let mut transposed_data = Vec::with_capacity(self.data.len());

        for col in 0..self.cols {
            for row in 0..self.rows {
                transposed_data.push(self[(row, col)].clone());
            }
        }

        Matrix {
            data: transposed_data,
            rows: self.cols,
            cols: self.rows,
        }
    }
}

impl<T: Float> Matrix<T> {
    pub fn standardize(&mut self) {
        for col in 0..self.cols {
            let mut sum = T::zero();
            for row in 0..self.rows {
                sum = sum + self[(row, col)];
            }
            let mean = sum / T::from(self.rows).unwrap();

            let mut variance_sum = T::zero();
            for row in 0..self.rows {
                let diff = self[(row, col)] - mean;
                variance_sum = variance_sum + diff * diff;
            }
            let variance = variance_sum / T::from(self.rows).unwrap();
            let std_dev = variance.sqrt();

            if std_dev <= T::zero() {
                continue;
            }

            for row in 0..self.rows {
                self[(row, col)] = (self[(row, col)] - mean) / std_dev;
            }
        }
    }

    pub fn normalize(&mut self) {
        for col in 0..self.cols {
            let mut min = self[(0, col)];
            let mut max = self[(0, col)];

            for row in 1..self.rows {
                if self[(row, col)] < min {
                    min = self[(row, col)];
                }
                if self[(row, col)] > max {
                    max = self[(row, col)];
                }
            }

            let range = max - min;

            if range <= T::zero() {
                continue;
            }

            for row in 0..self.rows {
                self[(row, col)] = (self[(row, col)] - min) / range;
            }
        }
    }
}

impl<T> AsRef<[T]> for Matrix<T> {
    fn as_ref(&self) -> &[T] {
        &self.data
    }
}

impl<T> Index<usize> for Matrix<T> {
    type Output = [T];

    fn index(&self, row: usize) -> &Self::Output {
        self.row(row)
    }
}

impl<T> Index<(usize, usize)> for Matrix<T> {
    type Output = T;

    fn index(&self, (row, col): (usize, usize)) -> &Self::Output {
        let flat_index = row * self.cols + col;
        &self.data[flat_index]
    }
}

impl<T> Index<[usize; 2]> for Matrix<T> {
    type Output = T;

    fn index(&self, index: [usize; 2]) -> &Self::Output {
        let flat_index = index[0] * self.cols + index[1];
        &self.data[flat_index]
    }
}

impl<T> IndexMut<usize> for Matrix<T> {
    fn index_mut(&mut self, row: usize) -> &mut Self::Output {
        self.row_mut(row)
    }
}

impl<T> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut Self::Output {
        let flat_index = row * self.cols + col;
        &mut self.data[flat_index]
    }
}

impl<T> IndexMut<[usize; 2]> for Matrix<T> {
    fn index_mut(&mut self, index: [usize; 2]) -> &mut Self::Output {
        let flat_index = index[0] * self.cols + index[1];
        &mut self.data[flat_index]
    }
}

impl<T: Default + Clone> From<(usize, usize)> for Matrix<T> {
    fn from((rows, cols): (usize, usize)) -> Self {
        let data = vec![T::default(); rows * cols];
        Matrix { data, rows, cols }
    }
}

impl<T> From<Vec<T>> for Matrix<T> {
    fn from(vec: Vec<T>) -> Self {
        let rows = vec.len();
        let cols = 1;
        Matrix {
            data: vec,
            rows,
            cols,
        }
    }
}

impl<T> From<Vec<Vec<T>>> for Matrix<T> {
    fn from(vec_of_vecs: Vec<Vec<T>>) -> Self {
        let rows = vec_of_vecs.len();
        let cols = if rows > 0 { vec_of_vecs[0].len() } else { 0 };
        let mut data = Vec::with_capacity(rows * cols);

        for row in vec_of_vecs.into_iter() {
            assert!(
                row.len() == cols,
                "All rows must have the same number of columns"
            );
            data.extend(row);
        }

        Matrix { data, rows, cols }
    }
}

impl<T> From<(usize, usize, Vec<T>)> for Matrix<T> {
    fn from((rows, cols, data): (usize, usize, Vec<T>)) -> Self {
        assert!(
            rows * cols == data.len(),
            "Data length must match rows * cols"
        );
        Matrix { data, rows, cols }
    }
}

impl<T> FromIterator<Vec<T>> for Matrix<T> {
    fn from_iter<I: IntoIterator<Item = Vec<T>>>(iter: I) -> Self {
        Matrix::from_rows(iter)
    }
}

impl<T: Clone> Clone for Matrix<T> {
    fn clone(&self) -> Self {
        Matrix {
            data: self.data.clone(),
            rows: self.rows,
            cols: self.cols,
        }
    }
}

impl<T: Default> Default for Matrix<T> {
    fn default() -> Self {
        Matrix {
            data: Vec::new(),
            rows: 0,
            cols: 0,
        }
    }
}

impl<T: PartialEq> PartialEq for Matrix<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows && self.cols == other.cols && self.data == other.data
    }
}

impl<T: Debug> Debug for Matrix<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        const MAX_ROWS: usize = 8;
        const MAX_COLS: usize = 8;

        writeln!(f, "Matrix({} × {}) {{", self.rows, self.cols)?;

        let rows = self.rows.min(MAX_ROWS);
        let cols = self.cols.min(MAX_COLS);

        for row in 0..rows {
            write!(f, "    [")?;

            for col in 0..cols {
                if col > 0 {
                    write!(f, ", ")?;
                }

                write!(f, "{:?}", self[(row, col)])?;
            }

            if cols < self.cols {
                write!(f, ", ...")?;
            }

            writeln!(f, "]")?;
        }

        if rows < self.rows {
            writeln!(f, "    ...")?;
        }

        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_indexing() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let matrix = Matrix::new(data).reshape(2, 3);

        assert_eq!(matrix[(0, 0)], 1);
        assert_eq!(matrix[(0, 1)], 2);
        assert_eq!(matrix[(1, 0)], 4);
        assert_eq!(matrix[[1, 2]], 6);
    }

    #[test]
    fn test_matrix_row_access() {
        let data = vec![1, 2, 3, 4, 5, 6];
        let matrix = Matrix::new(data).reshape(2, 3);

        assert_eq!(matrix.row(0), &[1, 2, 3]);
        assert_eq!(matrix.row(1), &[4, 5, 6]);
    }

    #[test]
    fn test_matrix_append_row() {
        let mut matrix = Matrix::new(vec![1, 2, 3, 4, 5, 6]).reshape(2, 3);
        matrix.append_row(vec![7, 8, 9]);

        assert_eq!(matrix.rows(), 3);
        assert_eq!(matrix.row(2), &[7, 8, 9]);
    }

    #[test]
    fn test_matrix_append_column() {
        let mut matrix = Matrix::new([1, 2, 3, 4]).reshape(2, 2);
        matrix.append_column(vec![5, 6]);

        assert_eq!(matrix.cols(), 3);
        assert_eq!(matrix.row(0), &[1, 2, 5]);
        assert_eq!(matrix.row(1), &[3, 4, 6]);
    }

    #[test]
    fn test_matrix_reshape() {
        let matrix = Matrix::new(vec![1, 2, 3, 4, 5, 6]).reshape(3, 2);

        assert_eq!(matrix.rows(), 3);
        assert_eq!(matrix.cols(), 2);
        assert_eq!(matrix.row(0), &[1, 2]);
        assert_eq!(matrix.row(1), &[3, 4]);
        assert_eq!(matrix.row(2), &[5, 6]);
    }

    #[test]
    fn test_matrix_split_at_row() {
        let matrix = Matrix::new(vec![1, 2, 3, 4, 5, 6, 7, 8]).reshape(4, 2);
        let (first_part, second_part) = matrix.split_at_row(2);

        assert_eq!(first_part.rows(), 2);
        assert_eq!(first_part.cols(), 2);
        assert_eq!(first_part.row(0), &[1, 2]);
        assert_eq!(first_part.row(1), &[3, 4]);
        assert_eq!(second_part.rows(), 2);
        assert_eq!(second_part.cols(), 2);
        assert_eq!(second_part.row(0), &[5, 6]);
        assert_eq!(second_part.row(1), &[7, 8]);
    }
}
