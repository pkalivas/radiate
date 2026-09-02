use radiate_core::random_provider;
use radiate_utils::{Float, Matrix};

#[derive(Default, Clone)]
pub struct DataSet<T> {
    features: Matrix<T>,
    labels: Matrix<T>,
}

impl<T> DataSet<T> {
    pub fn new(inputs: Vec<Vec<T>>, outputs: Vec<Vec<T>>) -> Self {
        let features = Matrix::from(inputs);
        let labels = Matrix::from(outputs);

        DataSet { features, labels }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&[T], &[T])> {
        self.features.iter().zip(self.labels.iter())
    }

    pub fn len(&self) -> usize {
        self.features.rows()
    }

    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }

    pub fn shape(&self) -> (usize, usize, usize) {
        let num_samples = self.features.rows();
        let input_dim = if num_samples > 0 {
            self.features.cols()
        } else {
            0
        };
        let output_dim = if num_samples > 0 {
            self.labels.cols()
        } else {
            0
        };

        (num_samples, input_dim, output_dim)
    }

    pub fn append(mut self, features: Vec<T>, labels: Vec<T>) -> Self {
        self.features.append_row(features);
        self.labels.append_row(labels);
        self
    }
}

impl<T: Clone> DataSet<T> {
    #[inline]
    pub fn features(&self) -> Vec<Vec<T>> {
        (0..self.features.rows())
            .map(|i| self.features.row(i).to_vec())
            .collect()
    }

    #[inline]
    pub fn labels(&self) -> Vec<Vec<T>> {
        (0..self.labels.rows())
            .map(|i| self.labels.row(i).to_vec())
            .collect()
    }

    pub fn shuffle(self) -> Self {
        let mut indices: Vec<usize> = (0..self.len()).collect();
        random_provider::shuffle(&mut indices);

        let features = self.features.sort_by_indices(&indices);
        let labels = self.labels.sort_by_indices(&indices);

        DataSet { features, labels }
    }

    #[inline]
    pub fn split(self, ratio: f32) -> (Self, Self) {
        let ratio = ratio.clamp(0.0, 1.0);
        let split = (self.len() as f32 * ratio).round() as usize;
        let (features_left, features_right) = self.features.split_at_row(split);
        let (labels_left, labels_right) = self.labels.split_at_row(split);

        (
            DataSet {
                features: features_left,
                labels: labels_left,
            },
            DataSet {
                features: features_right,
                labels: labels_right,
            },
        )
    }
}

impl<F: Float> DataSet<F> {
    pub fn standardize(mut self) -> Self {
        self.features.standardize();
        self
    }

    pub fn normalize(mut self) -> Self {
        self.features.normalize();
        self
    }
}

impl<T> From<(Vec<Vec<T>>, Vec<Vec<T>>)> for DataSet<T> {
    fn from(data: (Vec<Vec<T>>, Vec<Vec<T>>)) -> Self {
        DataSet::new(data.0, data.1)
    }
}

impl<T> From<(Matrix<T>, Matrix<T>)> for DataSet<T> {
    fn from(data: (Matrix<T>, Matrix<T>)) -> Self {
        DataSet {
            features: data.0,
            labels: data.1,
        }
    }
}
