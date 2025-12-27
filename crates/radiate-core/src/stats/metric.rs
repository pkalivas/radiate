use super::Statistic;
use crate::{
    TimeStatistic,
    stats::{Tag, TagKind, defaults},
};
use radiate_utils::{ToSnakeCase, cache_arc_string, intern, intern_snake_case};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};

#[macro_export]
macro_rules! metric {
    ($name:expr, $update:expr) => {{
        let mut metric = $crate::Metric::new($name);
        metric.apply_update($update);
        metric
    }};
    ($name:expr) => {{ $crate::Metric::new($name).upsert(1) }};
}

#[derive(Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MetricInner {
    pub(crate) value_statistic: Option<Statistic>,
    pub(crate) time_statistic: Option<TimeStatistic>,
}

#[derive(Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Metric {
    pub(super) name: Arc<String>,
    pub(super) inner: MetricInner,
    pub(super) tags: Tag,
}

impl Metric {
    pub fn new(name: &'static str) -> Self {
        let name = cache_arc_string!(intern_snake_case!(name));
        let tags = defaults::default_tags(&name);

        Self {
            name,
            inner: MetricInner {
                value_statistic: None,
                time_statistic: None,
            },
            tags,
        }
    }

    #[inline(always)]
    pub fn tags(&self) -> Tag {
        self.tags
    }

    #[inline(always)]
    pub fn with_tag(mut self, tag: TagKind) -> Self {
        self.add_tag(tag);
        self
    }

    #[inline(always)]
    pub fn with_tags<T>(&mut self, tags: T)
    where
        T: Into<Tag>,
    {
        self.tags = tags.into();
    }

    #[inline(always)]
    pub fn add_tag(&mut self, tag: TagKind) {
        self.tags.insert(tag);
    }

    pub fn contains_tag(&self, tag: &TagKind) -> bool {
        self.tags.has(*tag)
    }

    pub fn tags_iter(&self) -> impl Iterator<Item = TagKind> {
        self.tags.iter()
    }

    pub fn clear_values(&mut self) {
        self.inner = MetricInner::default();
    }

    #[inline(always)]
    pub fn upsert<'a>(mut self, update: impl Into<MetricUpdate<'a>>) -> Self {
        self.apply_update(update);
        self
    }

    #[inline(always)]
    pub fn update_from(&mut self, other: Metric) {
        if let Some(stat) = other.inner.value_statistic {
            // Kinda a hack to take advantage of the fact that if count == sum,
            // we can just apply the sum directly instead of merging statistics - keeps things honest
            if stat.count() as f32 == stat.sum() && !other.tags.has(TagKind::Distribution) {
                self.apply_update(stat.sum());
            } else {
                self.apply_update(stat);
            }
        }

        if let Some(time) = other.inner.time_statistic {
            self.apply_update(time);
        }

        self.tags = self.tags.union(other.tags);
    }

    #[inline(always)]
    pub fn apply_update<'a>(&mut self, update: impl Into<MetricUpdate<'a>>) {
        let update = update.into();
        match update {
            MetricUpdate::Float(value) => {
                self.update_statistic(value);
            }
            MetricUpdate::Usize(value) => {
                self.update_statistic(value as f32);
            }
            MetricUpdate::Duration(value) => {
                self.update_time_statistic(value);
            }
            MetricUpdate::FloatOperation(value, time) => {
                self.update_statistic(value);
                self.update_time_statistic(time);
            }
            MetricUpdate::UsizeOperation(value, time) => {
                self.update_statistic(value as f32);
                self.update_time_statistic(time);
            }
            MetricUpdate::UsizeDistribution(values) => {
                self.update_statistic_from_iter(values.iter().map(|v| *v as f32));
            }
            MetricUpdate::Distribution(values) => {
                self.update_statistic_from_iter(values.iter().cloned());
            }
            MetricUpdate::Statistic(stat) => {
                if let Some(existing_stat) = &mut self.inner.value_statistic {
                    existing_stat.merge(&stat);
                } else {
                    self.new_statistic(stat);
                }
            }
            MetricUpdate::TimeStatistic(time_stat) => {
                if let Some(existing_time_stat) = &mut self.inner.time_statistic {
                    existing_time_stat.merge(&time_stat);
                } else {
                    self.new_time_statistic(time_stat);
                }
            }
        }
    }

    pub fn new_statistic(&mut self, value: impl Into<Statistic>) {
        self.inner.value_statistic = Some(value.into());
        self.add_tag(TagKind::Statistic);
    }

    pub fn new_time_statistic(&mut self, value: impl Into<TimeStatistic>) {
        self.inner.time_statistic = Some(value.into());
        self.add_tag(TagKind::Time);
    }

    fn update_statistic(&mut self, value: f32) {
        if let Some(stat) = &mut self.inner.value_statistic {
            stat.add(value);
        } else {
            self.new_statistic(value);
        }
    }

    fn update_time_statistic(&mut self, value: Duration) {
        if let Some(stat) = &mut self.inner.time_statistic {
            stat.add(value);
        } else {
            self.new_time_statistic(value);
        }
    }

    fn update_statistic_from_iter<I>(&mut self, values: I)
    where
        I: IntoIterator<Item = f32>,
    {
        if let Some(stat) = &mut self.inner.value_statistic {
            for value in values {
                stat.add(value);
            }
        } else {
            let mut new_stat = Statistic::default();
            for value in values {
                new_stat.add(value);
            }

            self.new_statistic(new_stat);
            self.add_tag(TagKind::Distribution);
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
        if let Some(stat) = &self.inner.value_statistic {
            return stat.count();
        } else if let Some(stat) = &self.inner.time_statistic {
            return stat.count();
        }

        // No statistics recorded yet
        0
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

    pub fn value_count(&self) -> Option<i32> {
        self.statistic().map(|stat| stat.count())
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
}

#[derive(Clone, PartialEq)]
pub enum MetricUpdate<'a> {
    Float(f32),
    Usize(usize),
    Duration(Duration),
    FloatOperation(f32, Duration),
    UsizeOperation(usize, Duration),
    Distribution(&'a [f32]),
    UsizeDistribution(&'a [usize]),
    Statistic(Statistic),
    TimeStatistic(TimeStatistic),
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
        MetricUpdate::Distribution(value)
    }
}

impl<'a> From<&'a Vec<usize>> for MetricUpdate<'a> {
    fn from(value: &'a Vec<usize>) -> Self {
        MetricUpdate::UsizeDistribution(value)
    }
}

impl From<Statistic> for MetricUpdate<'_> {
    fn from(value: Statistic) -> Self {
        MetricUpdate::Statistic(value)
    }
}

impl From<TimeStatistic> for MetricUpdate<'_> {
    fn from(value: TimeStatistic) -> Self {
        MetricUpdate::TimeStatistic(value)
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
