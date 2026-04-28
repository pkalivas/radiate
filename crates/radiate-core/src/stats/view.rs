use radiate_utils::Statistic;

pub struct MetricView<'a, T> {
    pub(super) name: &'a str,
    pub(super) statistic: &'a Statistic,
    pub(super) mapper: fn(f32) -> T,
}

impl<'a, T> MetricView<'a, T> {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn count(&self) -> i32 {
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
}
