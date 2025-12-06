use super::Statistic;
use crate::{Distribution, TimeStatistic, stats::defaults};
use radiate_utils::{ToSnakeCase, cache_string, intern, intern_snake_case};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::Arc, time::Duration};

#[macro_export]
macro_rules! metric {
    ($name:expr, $update:expr) => {{
        let mut metric = $crate::Metric::new($name);
        metric.apply_update($update);
        metric
    }};
    ($scope:expr, $name:expr, $value:expr) => {{ $crate::Metric::new_scoped($name).upsert($value) }};
    ($name:expr) => {{ $crate::Metric::new($name).upsert(1) }};
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

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct Tag(pub &'static str);

impl From<&'static str> for Tag {
    fn from(value: &'static str) -> Self {
        Tag(intern_snake_case!(value))
    }
}

#[derive(Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MetricInner {
    pub(crate) value_statistic: Option<Statistic>,
    pub(crate) time_statistic: Option<TimeStatistic>,
    pub(crate) distribution: Option<Distribution>,
}

#[derive(Clone, PartialEq, Default)]
pub struct Metric {
    pub(super) name: Arc<String>,
    pub(super) inner: MetricInner,
    pub(super) rollup: Rollup,
    pub(super) tags: Option<Arc<Vec<Tag>>>,
}

impl Metric {
    pub fn new(name: &'static str) -> Self {
        let name = cache_string!(intern_snake_case!(name));
        let rollup = defaults::default_rollup(&name);
        let tags = defaults::default_tags(&name);

        Self {
            name,
            inner: MetricInner {
                value_statistic: None,
                time_statistic: None,
                distribution: None,
            },
            rollup,
            tags: tags,
        }
    }

    pub fn new_scoped(name: &'static str) -> Self {
        let rollup = defaults::default_rollup(name);

        Self {
            rollup,
            ..Self::new(intern_snake_case!(name))
        }
    }

    pub fn with_rollup(mut self, rollup: Rollup) -> Self {
        self.rollup = rollup;
        self
    }

    pub fn with_tag(mut self, tag: &'static str) -> Self {
        self.add_tag(tag);
        self
    }

    pub fn with_tags(mut self, tags: Vec<&'static str>) -> Self {
        let arc_tags = tags.into_iter().map(|tag| Tag(intern!(tag))).collect();
        self.tags = Some(Arc::new(arc_tags));
        self
    }

    pub fn add_tag<'a>(&mut self, tag: impl Into<Tag>) {
        let tag = tag.into();
        match &mut self.tags {
            Some(tags) => {
                if !tags.iter().any(|t| t.0 == tag.0) {
                    Arc::make_mut(tags).push(tag);
                }
            }
            None => {
                self.tags = Some(Arc::new(vec![tag]));
            }
        }
    }

    pub fn inner(&self) -> &MetricInner {
        &self.inner
    }

    pub fn rollup(&self) -> Rollup {
        self.rollup
    }

    pub fn tags(&self) -> Option<&[Tag]> {
        self.tags.as_deref().map(|v| v.as_slice())
    }

    pub fn contains_tag(&self, tag: &Tag) -> bool {
        if let Some(tags) = &self.tags {
            tags.iter().any(|t| t.0 == tag.0)
        } else {
            false
        }
    }

    pub fn get_stat<F, T>(&self, func: F) -> T
    where
        F: Fn(&Statistic) -> T,
        T: Default,
    {
        self.inner
            .value_statistic
            .as_ref()
            .map(|stat| func(stat))
            .unwrap_or_default()
    }

    pub fn get_dist<F, T>(&self, func: F) -> T
    where
        F: Fn(&Distribution) -> T,
        T: Default,
    {
        self.inner
            .distribution
            .as_ref()
            .map(|dist| func(dist))
            .unwrap_or_default()
    }

    pub fn get_time<F, T>(&self, func: F) -> T
    where
        F: Fn(&TimeStatistic) -> T,
        T: Default,
    {
        self.inner
            .time_statistic
            .as_ref()
            .map(|time| func(time))
            .unwrap_or_default()
    }

    pub fn clear_values(&mut self) {
        self.inner = MetricInner::default();
    }

    #[inline(always)]
    pub fn upsert<'a>(mut self, update: impl Into<MetricUpdate<'a>>) -> Self {
        self.apply_update(update);
        self
    }

    pub fn update_from(&mut self, other: Metric) {
        if let Some(stat) = &other.inner.value_statistic {
            let v = (stat.last_value(), stat.min(), stat.max(), stat.mean());
            match self.rollup() {
                Rollup::Sum => self.apply_update(stat.sum()),
                Rollup::Mean => self.apply_update(v.3),
                Rollup::Last => self.apply_update(v.0),
                Rollup::Min => self.apply_update(v.1),
                Rollup::Max => self.apply_update(v.2),
            }

            self.add_tag(defaults::metric_tags::STATISTIC);
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

            self.add_tag(defaults::metric_tags::TIME);
        }

        if let Some(d) = &other.inner.distribution {
            self.apply_update(d.last_sequence());
            self.add_tag(defaults::metric_tags::DISTRIBUTION);
        }

        if let Some(other_tags) = &other.tags {
            for tag in other_tags.iter() {
                self.add_tag(tag.0);
            }
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
            // MetricUpdate::UsizeOperationTagged(value, time, tag) => {
            //     if let Some(stat) = &mut self.inner.value_statistic {
            //         stat.add(value as f32);
            //     } else {
            //         self.inner.value_statistic = Some(Statistic::from(value as f32));
            //     }

            //     if let Some(time_stat) = &mut self.inner.time_statistic {
            //         time_stat.add(time);
            //     } else {
            //         self.inner.time_statistic = Some(TimeStatistic::from(time));
            //     }

            //     self.add_tag(tag);
            // }
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
    pub fn name(&self) -> &str {
        &self.name
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

    pub fn value_sum(&self) -> Option<f32> {
        self.statistic().map(|stat| stat.sum())
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
    pub fn last_sequence(&self) -> Option<&[f32]> {
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

// impl<'a> From<&'a Metric> for MetricUpdate<'a> {
//     fn from(value: &'a Metric) -> Self {
//         MetricUpdate::Metric(value)
//     }
// }

// impl<'a, T> From<(usize, Duration, T)> for MetricUpdate<'a>
// where
//     T: Into<Tag>,
// {
//     fn from(value: (usize, Duration, T)) -> Self {
//         MetricUpdate::UsizeOperationTagged(value.0, value.1, value.2.into())
//     }
// }

// impl<'a, T> From<(usize, Duration, Vec<T>)> for MetricUpdate<'a>
// where
//     T: Into<Tag>,
// {
//     fn from(value: (usize, Duration, Vec<T>)) -> Self {
//         let tags = value.2.into_iter().map(|t| t.into()).collect();
//         MetricUpdate::UsizeOperationTaggedMany(value.0, value.1, tags)
//     }
// }

impl std::fmt::Debug for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Metric {{ name: {}, }}", self.name)
    }
}

#[cfg(feature = "serde")]
impl Serialize for Metric {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use std::sync::Arc;

        #[derive(Serialize)]
        struct MetricOwned {
            name: String,
            inner: MetricInner,
            // scope: MetricScope,
            rollup: Rollup,
            tags: Option<Arc<Vec<Arc<String>>>>,
        }

        let tags = self
            .tags
            .as_ref()
            .map(|tags| Arc::new(tags.iter().map(|tag| Arc::new(tag.0.to_string())).collect()));
        let metric = MetricOwned {
            name: self.name.to_string(),
            inner: self.inner.clone(),
            // scope: self.scope,
            rollup: self.rollup,
            tags,
        };

        metric.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Metric {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::sync::Arc;

        use crate::stats::MetricInner;

        #[derive(Deserialize)]
        struct MetricOwned {
            name: String,
            inner: MetricInner,
            rollup: Rollup,
            tags: Option<Arc<Vec<Arc<String>>>>,
        }

        let metric = MetricOwned::deserialize(deserializer)?;

        Ok(Metric {
            name: cache_string!(intern_snake_case!(metric.name.as_str())),
            inner: metric.inner,
            rollup: metric.rollup,
            tags: metric
                .tags
                .map(|tags| Arc::new(tags.iter().map(|tag| Tag(intern!(tag.as_str()))).collect())),
        })
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
