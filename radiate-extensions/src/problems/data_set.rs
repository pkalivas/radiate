#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct Row<T>(pub usize, pub Vec<T>, pub Vec<T>);

#[derive(Default)]
pub struct DataSet<T> {
    samples: Vec<Row<T>>,
}

impl<T> DataSet<T> {
    pub fn new() -> Self {
        DataSet {
            samples: Vec::new(),
        }
    }

    pub fn from_samples(samples: Vec<Row<T>>) -> Self {
        DataSet { samples }
    }

    pub fn from_vecs(inputs: Vec<Vec<T>>, outputs: Vec<Vec<T>>) -> Self {
        let mut samples = Vec::new();
        for (input, output) in inputs.into_iter().zip(outputs.into_iter()) {
            samples.push(Row(samples.len(), input, output));
        }
        DataSet { samples }
    }

    pub fn add_sample(&mut self, input: Vec<T>, output: Vec<T>) {
        let index = self.samples.len();
        self.samples.push(Row(index, input, output));
    }

    pub fn get_sample(&self, index: usize) -> Option<&Row<T>> {
        self.samples.get(index)
    }

    pub fn get_samples(&self) -> &[Row<T>] {
        &self.samples
    }

    pub fn get_samples_mut(&mut self) -> &mut [Row<T>] {
        &mut self.samples
    }
}
