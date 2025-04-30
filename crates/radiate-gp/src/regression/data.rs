use radiate_core::random_provider;

#[derive(Debug, Clone, Default)]
pub struct Row {
    input: Vec<f32>,
    output: Vec<f32>,
}

impl Row {
    pub fn new(input: Vec<f32>, output: Vec<f32>) -> Self {
        Row { input, output }
    }

    pub fn input(&self) -> &Vec<f32> {
        &self.input
    }

    pub fn output(&self) -> &Vec<f32> {
        &self.output
    }
}

#[derive(Default, Clone)]
pub struct DataSet {
    rows: Vec<Row>,
}

impl DataSet {
    pub fn new(inputs: Vec<Vec<f32>>, outputs: Vec<Vec<f32>>) -> Self {
        let mut samples = Vec::new();
        for (input, output) in inputs.into_iter().zip(outputs.into_iter()) {
            samples.push(Row { input, output });
        }
        DataSet { rows: samples }
    }

    pub fn iter(&self) -> std::slice::Iter<Row> {
        self.rows.iter()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn shuffle(mut self) -> Self {
        random_provider::shuffle(&mut self.rows);
        self
    }

    pub fn features(&self) -> Vec<Vec<f32>> {
        self.rows.iter().map(|row| row.input.clone()).collect()
    }

    pub fn labels(&self) -> Vec<Vec<f32>> {
        self.rows.iter().map(|row| row.output.clone()).collect()
    }

    pub fn split(self, ratio: f32) -> (Self, Self) {
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

impl From<(Vec<Vec<f32>>, Vec<Vec<f32>>)> for DataSet {
    fn from(data: (Vec<Vec<f32>>, Vec<Vec<f32>>)) -> Self {
        DataSet::new(data.0, data.1)
    }
}
