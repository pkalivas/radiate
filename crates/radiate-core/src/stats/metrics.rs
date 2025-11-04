use super::Statistic;
use crate::{
    Distribution, TimeStatistic, intern,
    stats::{ToSnakeCase, defaults},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::Duration};

#[macro_export]
macro_rules! metric {
    ($name:expr, $update:expr) => {{
        let mut metric = $crate::Metric::new($name);
        metric.apply_update($update);
        metric
    }};
    ($scope:expr, $name:expr, $value:expr) => {{ $crate::Metric::new_scoped($name, $scope).upsert($value) }};
    ($name:expr) => {{ $crate::Metric::new($name).upsert(1) }};
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MetricScope {
    #[default]
    Generation,
    Lifetime,
    Step,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Rollup {
    #[default]
    Sum,
    Mean,
    Last,
    Min,
    Max,
}

#[derive(Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MetricInner {
    pub(crate) value_statistic: Option<Statistic>,
    pub(crate) time_statistic: Option<TimeStatistic>,
    pub(crate) distribution: Option<Distribution>,
}

#[derive(Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Metric {
    pub(super) name: &'static str,
    pub(super) inner: MetricInner,
    pub(super) scope: MetricScope,
    pub(super) rollup: Rollup,
}

impl Metric {
    pub fn new(name: &'static str) -> Self {
        let name = intern!(name.to_snake_case());
        let scope = defaults::default_scope(name);
        let rollup = defaults::default_rollup(name);
        Self {
            name,
            inner: MetricInner {
                value_statistic: None,
                time_statistic: None,
                distribution: None,
            },
            scope,
            rollup,
        }
    }

    pub fn new_scoped(name: &'static str, scope: MetricScope) -> Self {
        let rollup = defaults::default_rollup(name);
        Self {
            scope,
            rollup,
            ..Self::new(intern!(name.to_snake_case()))
        }
    }

    pub fn with_rollup(mut self, rollup: Rollup) -> Self {
        self.rollup = rollup;
        self
    }

    pub fn scope(&self) -> MetricScope {
        self.scope
    }
    pub fn rollup(&self) -> Rollup {
        self.rollup
    }

    pub fn clear_values(&mut self) {
        self.inner = MetricInner::default();
    }

    pub fn inner(&self) -> &MetricInner {
        &self.inner
    }

    #[inline(always)]
    pub fn upsert<'a>(mut self, update: impl Into<MetricUpdate<'a>>) -> Self {
        self.apply_update(update);
        self
    }

    pub fn update_from(&mut self, other: &Metric) {
        if let Some(stat) = &other.inner.value_statistic {
            let v = (stat.last_value(), stat.min(), stat.max(), stat.mean());
            match self.rollup() {
                Rollup::Sum => self.apply_update(stat.sum()),
                Rollup::Mean => self.apply_update(v.3),
                Rollup::Last => self.apply_update(v.0),
                Rollup::Min => self.apply_update(v.1),
                Rollup::Max => self.apply_update(v.2),
            }
        }

        if let Some(time) = &other.inner.time_statistic {
            let t = (
                time.last_time(),
                time.min(),
                time.max(),
                time.mean(),
                time.sum(),
            );
            match self.rollup() {
                Rollup::Sum => self.apply_update(t.4),
                Rollup::Mean => self.apply_update(t.3),
                Rollup::Last => self.apply_update(t.0),
                Rollup::Min => self.apply_update(t.1),
                Rollup::Max => self.apply_update(t.2),
            }
        }

        // Distributions — append most recent sequence if present.
        if let Some(d) = &other.inner.distribution {
            self.apply_update(d.last_sequence().as_slice());
        }
    }

    #[inline(always)]
    pub fn apply_update<'a>(&mut self, update: impl Into<MetricUpdate<'a>>) {
        let update = update.into();
        match update {
            MetricUpdate::Float(value) => {
                if let Some(stat) = &mut self.inner.value_statistic {
                    stat.add(value);
                } else {
                    self.inner.value_statistic = Some(Statistic::from(value));
                }
            }
            MetricUpdate::Usize(value) => {
                if let Some(stat) = &mut self.inner.value_statistic {
                    stat.add(value as f32);
                } else {
                    self.inner.value_statistic = Some(Statistic::from(value as f32));
                }
            }
            MetricUpdate::Duration(value) => {
                if let Some(stat) = &mut self.inner.time_statistic {
                    stat.add(value);
                } else {
                    self.inner.time_statistic = Some(TimeStatistic::from(value));
                }
            }
            MetricUpdate::Distribution(values) => {
                if let Some(stat) = &mut self.inner.distribution {
                    stat.add(values);
                } else {
                    self.inner.distribution = Some(Distribution::from(values));
                }
            }
            MetricUpdate::FloatOperation(value, time) => {
                if let Some(stat) = &mut self.inner.value_statistic {
                    stat.add(value);
                } else {
                    self.inner.value_statistic = Some(Statistic::from(value));
                }

                if let Some(time_stat) = &mut self.inner.time_statistic {
                    time_stat.add(time);
                } else {
                    self.inner.time_statistic = Some(TimeStatistic::from(time));
                }
            }
            MetricUpdate::UsizeOperation(value, time) => {
                if let Some(stat) = &mut self.inner.value_statistic {
                    stat.add(value as f32);
                } else {
                    self.inner.value_statistic = Some(Statistic::from(value as f32));
                }

                if let Some(time_stat) = &mut self.inner.time_statistic {
                    time_stat.add(time);
                } else {
                    self.inner.time_statistic = Some(TimeStatistic::from(time));
                }
            }
            MetricUpdate::DistributionRef(values) => {
                if let Some(stat) = &mut self.inner.distribution {
                    stat.add(values);
                } else {
                    self.inner.distribution = Some(Distribution::from(values.as_slice()));
                }
            }
            MetricUpdate::DistributionOwned(values) => {
                if let Some(stat) = &mut self.inner.distribution {
                    stat.add(&values);
                } else {
                    self.inner.distribution = Some(Distribution::from(values.as_slice()));
                }
            }
        }
    }

    ///
    /// --- Common statistic getters ---
    ///
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn last_value(&self) -> f32 {
        self.inner
            .value_statistic
            .as_ref()
            .map_or(0.0, |stat| stat.last_value())
    }

    pub fn distribution(&self) -> Option<&Distribution> {
        self.inner.distribution.as_ref()
    }

    pub fn statistic(&self) -> Option<&Statistic> {
        self.inner.value_statistic.as_ref()
    }

    pub fn time_statistic(&self) -> Option<&TimeStatistic> {
        self.inner.time_statistic.as_ref()
    }

    pub fn last_time(&self) -> Duration {
        self.time_statistic()
            .map_or(Duration::ZERO, |stat| stat.last_time())
    }

    pub fn count(&self) -> i32 {
        self.statistic().map(|stat| stat.count()).unwrap_or(0)
    }

    ///
    /// --- Get the value statistics ---
    ///
    pub fn value_mean(&self) -> Option<f32> {
        self.statistic().map(|stat| stat.mean())
    }

    pub fn value_variance(&self) -> Option<f32> {
        self.statistic().map(|stat| stat.variance())
    }

    pub fn value_std_dev(&self) -> Option<f32> {
        self.statistic().map(|stat| stat.std_dev())
    }

    pub fn value_skewness(&self) -> Option<f32> {
        self.statistic().map(|stat| stat.skewness())
    }

    pub fn value_min(&self) -> Option<f32> {
        self.statistic().map(|stat| stat.min())
    }

    pub fn value_max(&self) -> Option<f32> {
        self.statistic().map(|stat| stat.max())
    }

    ///
    /// --- Get the time statistics ---
    ///
    pub fn time_mean(&self) -> Option<Duration> {
        self.time_statistic().map(|stat| stat.mean())
    }

    pub fn time_variance(&self) -> Option<Duration> {
        self.time_statistic().map(|stat| stat.variance())
    }

    pub fn time_std_dev(&self) -> Option<Duration> {
        self.time_statistic().map(|stat| stat.standard_deviation())
    }

    pub fn time_min(&self) -> Option<Duration> {
        self.time_statistic().map(|stat| stat.min())
    }

    pub fn time_max(&self) -> Option<Duration> {
        self.time_statistic().map(|stat| stat.max())
    }

    pub fn time_sum(&self) -> Option<Duration> {
        self.time_statistic().map(|stat| stat.sum())
    }

    ///
    /// --- Get the distribution statistics ---
    ///
    pub fn last_sequence(&self) -> Option<&Vec<f32>> {
        self.distribution().map(|dist| dist.last_sequence())
    }

    pub fn distribution_mean(&self) -> Option<f32> {
        self.distribution().map(|dist| dist.mean())
    }

    pub fn distribution_variance(&self) -> Option<f32> {
        self.distribution().map(|dist| dist.variance())
    }

    pub fn distribution_std_dev(&self) -> Option<f32> {
        self.distribution().map(|dist| dist.standard_deviation())
    }

    pub fn distribution_skewness(&self) -> Option<f32> {
        self.distribution().map(|dist| dist.skewness())
    }

    pub fn distribution_kurtosis(&self) -> Option<f32> {
        self.distribution().map(|dist| dist.kurtosis())
    }

    pub fn distribution_min(&self) -> Option<f32> {
        self.distribution().map(|dist| dist.min())
    }

    pub fn distribution_max(&self) -> Option<f32> {
        self.distribution().map(|dist| dist.max())
    }

    pub fn distribution_entropy(&self) -> Option<f32> {
        self.distribution().map(|dist| dist.entropy())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetricUpdate<'a> {
    Float(f32),
    Usize(usize),
    Duration(Duration),
    Distribution(&'a [f32]),
    DistributionRef(&'a Vec<f32>),
    DistributionOwned(Vec<f32>),
    FloatOperation(f32, Duration),
    UsizeOperation(usize, Duration),
}

impl From<f32> for MetricUpdate<'_> {
    fn from(value: f32) -> Self {
        MetricUpdate::Float(value)
    }
}

impl From<usize> for MetricUpdate<'_> {
    fn from(value: usize) -> Self {
        MetricUpdate::Usize(value)
    }
}

impl From<Duration> for MetricUpdate<'_> {
    fn from(value: Duration) -> Self {
        MetricUpdate::Duration(value)
    }
}

impl<'a> From<&'a [f32]> for MetricUpdate<'a> {
    fn from(value: &'a [f32]) -> Self {
        MetricUpdate::Distribution(value)
    }
}

impl From<(f32, Duration)> for MetricUpdate<'_> {
    fn from(value: (f32, Duration)) -> Self {
        MetricUpdate::FloatOperation(value.0, value.1)
    }
}

impl From<(usize, Duration)> for MetricUpdate<'_> {
    fn from(value: (usize, Duration)) -> Self {
        MetricUpdate::UsizeOperation(value.0, value.1)
    }
}

impl<'a> From<&'a Vec<f32>> for MetricUpdate<'a> {
    fn from(value: &'a Vec<f32>) -> Self {
        MetricUpdate::DistributionRef(value)
    }
}

impl From<Vec<f32>> for MetricUpdate<'_> {
    fn from(value: Vec<f32>) -> Self {
        MetricUpdate::DistributionOwned(value)
    }
}

impl std::fmt::Debug for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Metric {{ name: {}, }}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric() {
        let mut metric = Metric::new("test");
        metric.apply_update(1.0);
        metric.apply_update(2.0);
        metric.apply_update(3.0);
        metric.apply_update(4.0);
        metric.apply_update(5.0);

        assert_eq!(metric.count(), 5);
        assert_eq!(metric.last_value(), 5.0);
        assert_eq!(metric.value_mean().unwrap(), 3.0);
        assert_eq!(metric.value_variance().unwrap(), 2.5);
        assert_eq!(metric.value_std_dev().unwrap(), 1.5811388);
        assert_eq!(metric.value_min().unwrap(), 1.0);
        assert_eq!(metric.value_max().unwrap(), 5.0);
        assert_eq!(metric.name(), "test");
    }

    #[test]
    fn test_metric_labels() {
        let mut metric = Metric::new("test");

        metric.apply_update(1.0);
        metric.apply_update(2.0);
        metric.apply_update(3.0);
        metric.apply_update(4.0);
        metric.apply_update(5.0);

        assert_eq!(metric.count(), 5);
        assert_eq!(metric.last_value(), 5.0);
        assert_eq!(metric.value_mean().unwrap(), 3.0);
        assert_eq!(metric.value_variance().unwrap(), 2.5);
        assert_eq!(metric.value_std_dev().unwrap(), 1.5811388);
        assert_eq!(metric.value_min().unwrap(), 1.0);
        assert_eq!(metric.value_max().unwrap(), 5.0);
    }
}
