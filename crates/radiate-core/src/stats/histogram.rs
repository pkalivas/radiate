use crate::Distribution;

pub struct Histogram {
    pub bins: Vec<usize>,
    pub min: f32,
    pub max: f32,
    pub bin_size: f32,
}

impl Histogram {
    pub fn new(data: &[f32], num_bins: usize) -> Self {
        if data.is_empty() || num_bins == 0 {
            return Histogram {
                bins: vec![],
                min: 0.0,
                max: 0.0,
                bin_size: 0.0,
            };
        }

        let min = data.iter().cloned().fold(f32::INFINITY, |a, b| a.min(b));
        let max = data
            .iter()
            .cloned()
            .fold(f32::NEG_INFINITY, |a, b| a.max(b));

        let bin_size = (max - min) / num_bins as f32;
        let mut bins = vec![0; num_bins];

        for &value in data {
            if value >= min && value <= max {
                let bin_index = ((value - min) / bin_size).floor() as usize;
                let bin_index = if bin_index >= num_bins {
                    num_bins - 1
                } else {
                    bin_index
                };
                bins[bin_index] += 1;
            }
        }

        Histogram {
            bins,
            min,
            max,
            bin_size,
        }
    }

    pub fn bins(&self) -> &Vec<usize> {
        &self.bins
    }

    pub fn min(&self) -> f32 {
        self.min
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn bin_size(&self) -> f32 {
        self.bin_size
    }

    pub fn num_bins(&self) -> usize {
        self.bins.len()
    }

    pub fn total_count(&self) -> usize {
        self.bins.iter().sum()
    }

    pub fn clear(&mut self) {
        self.bins.clear();
        self.min = 0.0;
        self.max = 0.0;
        self.bin_size = 0.0;
    }

    pub fn add_data(&mut self, data: &[f32]) {
        if data.is_empty() || self.bins.is_empty() {
            return;
        }

        for &value in data {
            if value >= self.min && value <= self.max {
                let bin_index = ((value - self.min) / self.bin_size).floor() as usize;
                let bin_index = if bin_index >= self.bins.len() {
                    self.bins.len() - 1
                } else {
                    bin_index
                };
                self.bins[bin_index] += 1;
            }
        }
    }

    pub fn from_data(data: &[f32], num_bins: usize) -> Self {
        Histogram::new(data, num_bins)
    }

    pub fn from_distribution(dist: &Distribution, num_bins: usize) -> Self {
        Histogram::new(&dist.last_sequence, num_bins)
    }
}
