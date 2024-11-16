#[allow(dead_code)]
pub struct Sample<T>(pub usize, pub Vec<T>, pub Vec<T>);

pub struct SampleSet<T> {
    samples: Vec<Sample<T>>,
}

impl<T> SampleSet<T> {
    pub fn new() -> Self {
        SampleSet {
            samples: Vec::new(),
        }
    }

    pub fn from_samples(samples: Vec<Sample<T>>) -> Self {
        SampleSet { samples }
    }

    pub fn from_vecs(inputs: Vec<Vec<T>>, outputs: Vec<Vec<T>>) -> Self {
        let mut samples = Vec::new();
        for (input, output) in inputs.into_iter().zip(outputs.into_iter()) {
            samples.push(Sample(samples.len(), input, output));
        }
        SampleSet { samples }
    }

    pub fn add_sample(&mut self, input: Vec<T>, output: Vec<T>) {
        let index = self.samples.len();
        self.samples.push(Sample(index, input, output));
    }

    pub fn get_sample(&self, index: usize) -> Option<&Sample<T>> {
        self.samples.get(index)
    }

    pub fn get_samples(&self) -> &[Sample<T>] {
        &self.samples
    }

    pub fn get_samples_mut(&mut self) -> &mut [Sample<T>] {
        &mut self.samples
    }
}
