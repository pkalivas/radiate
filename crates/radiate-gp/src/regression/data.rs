use radiate_core::random_provider;

#[derive(Debug, Clone, Default)]
pub struct Row<T> {
    input: Vec<T>,
    output: Vec<T>,
}

impl<T> Row<T> {
    pub fn new(input: Vec<T>, output: Vec<T>) -> Self {
        Row { input, output }
    }

    pub fn input(&self) -> &[T] {
        &self.input
    }

    pub fn output(&self) -> &[T] {
        &self.output
    }
}

impl<T> From<(Vec<T>, Vec<T>)> for Row<T> {
    fn from(data: (Vec<T>, Vec<T>)) -> Self {
        Row::new(data.0, data.1)
    }
}

#[derive(Default, Clone)]
pub struct DataSet<T> {
    rows: Vec<Row<T>>,
}

impl<T> DataSet<T> {
    pub fn new(inputs: Vec<Vec<T>>, outputs: Vec<Vec<T>>) -> Self {
        let mut samples = Vec::new();
        for (input, output) in inputs.into_iter().zip(outputs.into_iter()) {
            samples.push(Row { input, output });
        }

        DataSet { rows: samples }
    }

    pub fn row(mut self, row: impl Into<Row<T>>) -> Self {
        self.rows.push(row.into());
        self
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Row<T>> {
        self.rows.iter()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn shuffle(mut self) -> Self {
        random_provider::shuffle(&mut self.rows);
        self
    }

    pub fn shape(&self) -> (usize, usize, usize) {
        let num_samples = self.rows.len();
        let input_dim = if num_samples > 0 {
            self.rows[0].input.len()
        } else {
            0
        };
        let output_dim = if num_samples > 0 {
            self.rows[0].output.len()
        } else {
            0
        };

        (num_samples, input_dim, output_dim)
    }

    #[inline]
    pub fn features(&self) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        self.rows.iter().map(|row| row.input.clone()).collect()
    }

    #[inline]
    pub fn labels(&self) -> Vec<Vec<T>>
    where
        T: Clone,
    {
        self.rows.iter().map(|row| row.output.clone()).collect()
    }

    #[inline]
    pub fn split(self, ratio: f32) -> (Self, Self)
    where
        T: Clone,
    {
        let ratio = ratio.clamp(0.0, 1.0);
        let split = (self.len() as f32 * ratio).round() as usize;
        let (left, right) = self.rows.split_at(split);

        (
            DataSet {
                rows: left.to_vec(),
            },
            DataSet {
                rows: right.to_vec(),
            },
        )
    }
}

impl DataSet<f32> {
    pub fn standardize(mut self) -> Self {
        let mut means = vec![0.0; self.rows[0].input.len()];
        let mut stds = vec![0.0; self.rows[0].input.len()];

        for sample in self.rows.iter() {
            for (i, &val) in sample.input.iter().enumerate() {
                means[i] += val;
            }
        }

        let n = self.len() as f32;
        for mean in means.iter_mut() {
            *mean /= n;
        }

        for sample in self.rows.iter() {
            for (i, &val) in sample.input.iter().enumerate() {
                stds[i] += (val - means[i]).powi(2);
            }
        }

        for std in stds.iter_mut() {
            *std = (*std / n).sqrt();
        }

        for sample in self.rows.iter_mut() {
            for (i, val) in sample.input.iter_mut().enumerate() {
                *val = (*val - means[i]) / stds[i];
            }
        }

        self
    }

    pub fn normalize(mut self) -> Self {
        let mut mins = vec![f32::MAX; self.rows[0].input.len()];
        let mut maxs = vec![f32::MIN; self.rows[0].input.len()];

        for sample in self.rows.iter() {
            for (i, &val) in sample.input.iter().enumerate() {
                if val < mins[i] {
                    mins[i] = val;
                }

                if val > maxs[i] {
                    maxs[i] = val;
                }
            }
        }

        for sample in self.rows.iter_mut() {
            for (i, val) in sample.input.iter_mut().enumerate() {
                *val = (*val - mins[i]) / (maxs[i] - mins[i]);
            }
        }

        self
    }
}

impl<T> From<Vec<Vec<Option<T>>>> for DataSet<T>
where
    T: Clone,
{
    fn from(data: Vec<Vec<Option<T>>>) -> Self {
        let mut rows = Vec::new();
        for row in data.into_iter() {
            let input = row
                .iter()
                .filter_map(|v| v.as_ref())
                .cloned()
                .collect::<Vec<T>>();

            rows.push(Row {
                input,
                output: Vec::new(),
            });
        }

        DataSet { rows }
    }
}

impl<T> From<(Vec<Vec<T>>, Vec<Vec<T>>)> for DataSet<T> {
    fn from(data: (Vec<Vec<T>>, Vec<Vec<T>>)) -> Self {
        DataSet::new(data.0, data.1)
    }
}
