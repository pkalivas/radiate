use radiate::random_provider;

#[derive(Debug, Clone, Default)]
pub struct Row(pub usize, pub Vec<f32>, pub Vec<f32>);

#[derive(Default)]
pub struct DataSet {
    samples: Vec<Row>,
}

impl DataSet {
    pub fn new(inputs: Vec<Vec<f32>>, outputs: Vec<Vec<f32>>) -> Self {
        let mut samples = Vec::new();
        for (input, output) in inputs.into_iter().zip(outputs.into_iter()) {
            samples.push(Row(samples.len(), input, output));
        }
        DataSet { samples }
    }

    pub fn iter(&self) -> &[Row] {
        &self.samples
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn shuffle(&mut self) {
        random_provider::shuffle(&mut self.samples);

        for (i, sample) in self.samples.iter_mut().enumerate() {
            sample.0 = i;
        }
    }

    pub fn split(&self, ratio: f32) -> (Self, Self) {
        let split = (self.len() as f32 * ratio).round() as usize;
        let (left, right) = self.samples.split_at(split);

        (
            DataSet {
                samples: left.to_vec(),
            },
            DataSet {
                samples: right.to_vec(),
            },
        )
    }

    pub fn standardize(&mut self) {
        let mut means = vec![0.0; self.samples[0].1.len()];
        let mut stds = vec![0.0; self.samples[0].1.len()];

        for sample in self.samples.iter() {
            for (i, &val) in sample.1.iter().enumerate() {
                means[i] += val;
            }
        }

        let n = self.len() as f32;
        for mean in means.iter_mut() {
            *mean /= n;
        }

        for sample in self.samples.iter() {
            for (i, &val) in sample.1.iter().enumerate() {
                stds[i] += (val - means[i]).powi(2);
            }
        }

        for std in stds.iter_mut() {
            *std = (*std / n).sqrt();
        }

        for sample in self.samples.iter_mut() {
            for (i, val) in sample.1.iter_mut().enumerate() {
                *val = (*val - means[i]) / stds[i];
            }
        }
    }

    pub fn normalize(&mut self) {
        let mut mins = vec![f32::MAX; self.samples[0].1.len()];
        let mut maxs = vec![f32::MIN; self.samples[0].1.len()];

        for sample in self.samples.iter() {
            for (i, &val) in sample.1.iter().enumerate() {
                if val < mins[i] {
                    mins[i] = val;
                }

                if val > maxs[i] {
                    maxs[i] = val;
                }
            }
        }

        for sample in self.samples.iter_mut() {
            for (i, val) in sample.1.iter_mut().enumerate() {
                *val = (*val - mins[i]) / (maxs[i] - mins[i]);
            }
        }
    }
}
