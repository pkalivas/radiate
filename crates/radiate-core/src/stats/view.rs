use radiate_utils::Statistic;

pub struct MetricView<'a, T> {
    pub(super) name: &'a str,
    pub(super) statistic: &'a Statistic,
    pub(super) mapper: fn(f32) -> Option<T>,
}

impl<'a, T> MetricView<'a, T> {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn count(&self) -> i32 {
        self.statistic.count()
    }

    pub fn last(&self) -> Option<T> {
        (self.mapper)(self.statistic.last_value())
    }

    pub fn sum(&self) -> Option<T> {
        (self.mapper)(self.statistic.sum())
    }

    pub fn mean(&self) -> Option<T> {
        (self.mapper)(self.statistic.mean())
    }

    pub fn var(&self) -> Option<T> {
        (self.mapper)(self.statistic.variance()?)
    }

    pub fn stddev(&self) -> Option<T> {
        (self.mapper)(self.statistic.std_dev()?)
    }

    pub fn skewness(&self) -> Option<T> {
        (self.mapper)(self.statistic.skewness()?)
    }

    pub fn kurtosis(&self) -> Option<T> {
        (self.mapper)(self.statistic.kurtosis()?)
    }

    pub fn min(&self) -> Option<T> {
        (self.mapper)(self.statistic.min())
    }

    pub fn max(&self) -> Option<T> {
        (self.mapper)(self.statistic.max())
    }
}
