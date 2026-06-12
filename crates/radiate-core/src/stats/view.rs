use radiate_utils::{Quantile, Statistic};

pub struct MetricView<'a, T> {
    pub(super) name: &'a str,
    pub(super) statistic: &'a Statistic,
    pub(super) samples: Option<&'a [f32]>,
    pub(super) mapper: fn(f32) -> T,
}

impl<'a, T> MetricView<'a, T> {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn count(&self) -> u32 {
        self.statistic.count()
    }

    pub fn last(&self) -> T {
        (self.mapper)(self.statistic.last_value())
    }

    pub fn sum(&self) -> T {
        (self.mapper)(self.statistic.sum())
    }

    pub fn mean(&self) -> T {
        (self.mapper)(self.statistic.mean())
    }

    pub fn var(&self) -> T {
        (self.mapper)(self.statistic.variance().unwrap_or_default())
    }

    pub fn stddev(&self) -> T {
        (self.mapper)(self.statistic.std_dev().unwrap_or_default())
    }

    pub fn skewness(&self) -> T {
        (self.mapper)(self.statistic.skewness().unwrap_or_default())
    }

    pub fn kurtosis(&self) -> T {
        (self.mapper)(self.statistic.kurtosis().unwrap_or_default())
    }

    pub fn min(&self) -> T {
        (self.mapper)(self.statistic.min())
    }

    pub fn max(&self) -> T {
        (self.mapper)(self.statistic.max())
    }

    pub fn quantile(&self, q: f32) -> Option<T> {
        if let Some(samples) = &self.samples {
            let mut quant = Quantile::new(q);
            for &value in samples.iter() {
                if !value.is_finite() {
                    continue;
                }

                quant.add(value);
            }

            quant.value().map(self.mapper)
        } else {
            None
        }
    }

    pub fn quantiles(&self, quantiles: &[f32]) -> Option<Vec<T>> {
        if let Some(samples) = &self.samples {
            let mut quants: Vec<Quantile> = quantiles.iter().map(|&q| Quantile::new(q)).collect();
            for &value in samples.iter() {
                if !value.is_finite() {
                    continue;
                }

                for quant in quants.iter_mut() {
                    quant.add(value);
                }
            }

            quants
                .iter()
                .map(|quant| quant.value().map(self.mapper))
                .collect()
        } else {
            None
        }
    }

    pub fn samples(&self) -> Option<&[f32]> {
        self.samples
    }
}
