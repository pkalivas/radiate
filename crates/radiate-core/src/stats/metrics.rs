use super::Statistic;
use crate::{Distribution, TimeStatistic};
use std::sync::OnceLock;
use std::{
    collections::{BTreeMap, HashSet},
    fmt::Debug,
    sync::Mutex,
    time::Duration,
};

static INTERNED: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();

pub fn intern(name: String) -> &'static str {
    let mut interned = INTERNED
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
        .unwrap();
    if let Some(&existing) = interned.get(&*name) {
        return existing;
    }

    let static_name: &'static str = Box::leak(name.into_boxed_str());
    interned.insert(static_name);
    static_name
}

#[macro_export]
macro_rules! metric {
    ($name:expr, $time:expr) => {{
        let mut metric = $crate::Metric::new($name);
        metric.apply_update($time);
        metric
    }};
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct MetricLabel {
    pub key: &'static str,
    pub value: String,
}

impl MetricLabel {
    pub fn new(key: &'static str, value: impl Into<String>) -> Self {
        Self {
            key,
            value: value.into(),
        }
    }
}

#[macro_export]
macro_rules! labels {
    ($($key:expr => $value:expr),* $(,)?) => {
        vec![
            $(
                $crate::stats::metrics::MetricLabel::new($key, $value)
            ),*
        ]
    };
}

#[derive(Default, Clone)]
pub struct MetricSet {
    metrics: BTreeMap<&'static str, Metric>,
}

impl MetricSet {
    pub fn new() -> Self {
        MetricSet {
            metrics: BTreeMap::new(),
        }
    }

    pub fn merge(&mut self, other: &MetricSet) {
        for (name, metric) in other.iter() {
            if let Some(existing_metric) = self.metrics.get_mut(name) {
                if let Some(value_stat) = &metric.inner.value_statistic {
                    if let Some(existing_value_stat) = &mut existing_metric.inner.value_statistic {
                        existing_value_stat.add(value_stat.last_value());
                    } else {
                        existing_metric.inner.value_statistic = Some(value_stat.clone());
                    }
                }

                if let Some(time_stat) = &metric.inner.time_statistic {
                    if let Some(existing_time_stat) = &mut existing_metric.inner.time_statistic {
                        existing_time_stat.add(time_stat.last_time());
                    } else {
                        existing_metric.inner.time_statistic = Some(time_stat.clone());
                    }
                }

                if let Some(dist) = &metric.inner.distribution {
                    if let Some(existing_dist) = &mut existing_metric.inner.distribution {
                        existing_dist.add(&dist.last_sequence());
                    } else {
                        existing_metric.inner.distribution = Some(dist.clone());
                    }
                }

                if let Some(labels) = &metric.labels {
                    existing_metric.labels = Some(labels.clone());
                }
            } else {
                self.add(metric.clone());
            }
        }
    }

    pub fn add_labels(&mut self, name: &'static str, labels: Vec<MetricLabel>) {
        if let Some(m) = self.metrics.get_mut(name) {
            for label in labels {
                m.add_label(label);
            }
        }
    }

    pub fn upsert<'a>(&mut self, name: &'static str, update: impl Into<MetricUpdate<'a>>) {
        if let Some(m) = self.metrics.get_mut(name) {
            m.apply_update(update);
        } else {
            self.add(Metric::new(name));
            self.upsert(name, update);
        }
    }

    pub fn add_or_update<'a>(&mut self, metric: Metric) {
        if let Some(m) = self.metrics.get_mut(metric.name()) {
            m.apply_update(metric.last_value());
        } else {
            self.add(metric);
        }
    }

    pub fn add(&mut self, metric: Metric) {
        self.metrics.insert(metric.name(), metric);
    }

    pub fn get(&self, name: &'static str) -> Option<&Metric> {
        self.metrics.get(name)
    }

    pub fn get_mut(&mut self, name: &'static str) -> Option<&mut Metric> {
        self.metrics.get_mut(name)
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.metrics.keys().copied().collect()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &Metric)> {
        self.metrics.iter().map(|(name, metric)| (*name, metric))
    }

    pub fn get_from_string(&self, name: String) -> Option<&Metric> {
        self.metrics.get(name.as_str())
    }

    pub fn clear(&mut self) {
        self.metrics.clear();
    }

    pub fn contains_key(&self, name: impl Into<String>) -> bool {
        self.metrics.contains_key(intern(name.into()))
    }
}

impl Debug for MetricSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MetricSet {{\n")?;
        write!(f, "{}", format_metrics_table(&self))?;
        write!(f, "}}")
    }
}

#[derive(Clone, PartialEq, Default)]
pub struct MetricInner {
    pub(crate) value_statistic: Option<Statistic>,
    pub(crate) time_statistic: Option<TimeStatistic>,
    pub(crate) distribution: Option<Distribution>,
}

#[derive(Clone, PartialEq, Default)]
pub struct Metric {
    name: &'static str,
    inner: MetricInner,
    labels: Option<HashSet<MetricLabel>>,
}

impl Metric {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            inner: MetricInner {
                value_statistic: None,
                time_statistic: None,
                distribution: None,
            },
            labels: None,
        }
    }

    pub fn inner(&self) -> &MetricInner {
        &self.inner
    }

    pub fn labels(&self) -> Option<&HashSet<MetricLabel>> {
        self.labels.as_ref()
    }

    pub fn with_labels(mut self, labels: Vec<MetricLabel>) -> Self {
        self.labels.get_or_insert_with(HashSet::new).extend(labels);
        self
    }

    pub fn add_label(&mut self, label: MetricLabel) {
        self.labels.get_or_insert_with(HashSet::new).insert(label);
    }

    pub fn upsert<'a>(mut self, update: impl Into<MetricUpdate<'a>>) -> Self {
        self.apply_update(update);
        self
    }

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

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn last_value(&self) -> f32 {
        self.inner
            .value_statistic
            .as_ref()
            .map_or(0.0, |stat| stat.last_value())
    }

    pub fn last_time(&self) -> Duration {
        self.inner
            .time_statistic
            .as_ref()
            .map_or(Duration::ZERO, |stat| stat.last_time())
    }

    pub fn value_mean(&self) -> Option<f32> {
        self.inner.value_statistic.as_ref().map(|stat| stat.mean())
    }

    pub fn value_variance(&self) -> Option<f32> {
        self.inner
            .value_statistic
            .as_ref()
            .map(|stat| stat.variance())
    }

    pub fn value_std_dev(&self) -> Option<f32> {
        self.inner
            .value_statistic
            .as_ref()
            .map(|stat| stat.std_dev())
    }

    pub fn value_skewness(&self) -> Option<f32> {
        self.inner
            .value_statistic
            .as_ref()
            .map(|stat| stat.skewness())
    }

    pub fn value_min(&self) -> Option<f32> {
        self.inner.value_statistic.as_ref().map(|stat| stat.min())
    }

    pub fn value_max(&self) -> Option<f32> {
        self.inner.value_statistic.as_ref().map(|stat| stat.max())
    }

    pub fn time_mean(&self) -> Option<Duration> {
        self.inner.time_statistic.as_ref().map(|stat| stat.mean())
    }

    pub fn time_variance(&self) -> Option<Duration> {
        self.inner
            .time_statistic
            .as_ref()
            .map(|stat| stat.variance())
    }

    pub fn time_std_dev(&self) -> Option<Duration> {
        self.inner
            .time_statistic
            .as_ref()
            .map(|stat| stat.standard_deviation())
    }

    pub fn time_min(&self) -> Option<Duration> {
        self.inner.time_statistic.as_ref().map(|stat| stat.min())
    }

    pub fn time_max(&self) -> Option<Duration> {
        self.inner.time_statistic.as_ref().map(|stat| stat.max())
    }

    pub fn time_sum(&self) -> Option<Duration> {
        self.inner.time_statistic.as_ref().map(|stat| stat.sum())
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

    pub fn last_sequence(&self) -> Option<&Vec<f32>> {
        self.inner
            .distribution
            .as_ref()
            .map(|dist| dist.last_sequence())
    }

    pub fn distribution_mean(&self) -> Option<f32> {
        self.inner.distribution.as_ref().map(|dist| dist.mean())
    }

    pub fn distribution_variance(&self) -> Option<f32> {
        self.inner.distribution.as_ref().map(|dist| dist.variance())
    }

    pub fn distribution_std_dev(&self) -> Option<f32> {
        self.inner
            .distribution
            .as_ref()
            .map(|dist| dist.standard_deviation())
    }

    pub fn distribution_skewness(&self) -> Option<f32> {
        self.inner.distribution.as_ref().map(|dist| dist.skewness())
    }

    pub fn distribution_kurtosis(&self) -> Option<f32> {
        self.inner.distribution.as_ref().map(|dist| dist.kurtosis())
    }

    pub fn distribution_min(&self) -> Option<f32> {
        self.inner.distribution.as_ref().map(|dist| dist.min())
    }

    pub fn distribution_max(&self) -> Option<f32> {
        self.inner.distribution.as_ref().map(|dist| dist.max())
    }

    pub fn count(&self) -> i32 {
        self.inner
            .value_statistic
            .as_ref()
            .map(|stat| stat.count())
            .unwrap_or(0)
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

pub fn format_metrics_table(metrics: &MetricSet) -> String {
    use std::fmt::Write;

    let mut grouped: std::collections::BTreeMap<&str, &Metric> = std::collections::BTreeMap::new();
    for metric in metrics.iter().map(|(_, m)| m) {
        grouped.insert(metric.name(), metric);
    }

    let mut output = String::new();
    writeln!(
        output,
        "{:<24} | {:<6} | {:<10} | {:<10} | {:<10} | {:<6} | {:<12} | {:<10} | {:<10} | {:<10} | {:<10}",
        "Name", "Type", "Mean", "Min", "Max", "N", "Total", "StdDev", "Skew", "Kurt", "Entr"
    )
    .unwrap();
    writeln!(output, "{}", "-".repeat(145)).unwrap();

    for (name, metric) in grouped {
        let inner = metric.inner();

        // Value row
        if let Some(stat) = &inner.value_statistic {
            writeln!(
                output,
                "{:<24} | {:<6} | {:<10.3} | {:<10.3} | {:<10.3} | {:<6} | {:<12} | {:<10.3} | {:<10.3} | {:<10.3} | {:<10.3}",
                name,
                "value",
                stat.mean(),
                stat.min(),
                stat.max(),
                stat.count(),
                "-",
                stat.std_dev(),
                stat.skewness(),
                stat.kurtosis(),
                "-",
            )
            .unwrap();
        }

        // Time row
        if let Some(time) = &inner.time_statistic {
            writeln!(
                output,
                "{:<24} | {:<6} | {:<10} | {:<10} | {:<10} | {:<6} | {:<12} | {:<10} | {:<10} | {:<10} | {:<10}",
                name,
                "time",
                format!("{:?}", time.mean()),
                format!("{:?}", time.min()),
                format!("{:?}", time.max()),
                time.count(),
                format!("{:?}", time.sum()),
                format!("{:?}", time.standard_deviation()),
                "-",
                "-",
                "-",

            )
            .unwrap();
        }

        // Distribution row
        if let Some(dist) = &inner.distribution {
            writeln!(
                output,
                "{:<24} | {:<6} | {:<10.3} | {:<10.3} | {:<10.3} | {:<6} | {:<12} | {:<10.3} | {:<10.3} | {:<10.3} | {:<10.3}",
                name,
                "dist",
                dist.mean(),
                dist.min(),
                dist.max(),
                dist.count(),
                format!("{:.3}", dist.entropy()),
                dist.standard_deviation(),
                dist.skewness(),
                dist.kurtosis(),
                format!("{:.3}", dist.entropy()),
            )
            .unwrap();
        }

        if let Some(labels) = &metric.labels {
            let labels_str = labels
                .iter()
                .map(|l| format!("{}={}", l.key, l.value))
                .collect::<Vec<String>>()
                .join(", ");
            writeln!(output, "{:<24} | Labels: {}", "", labels_str).unwrap();
        }
    }

    output
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
    fn test_metric_set() {
        let mut metric_set = MetricSet::new();
        metric_set.upsert("test", 1.0);
        metric_set.upsert("test", 2.0);
        metric_set.upsert("test", 3.0);
        metric_set.upsert("test", 4.0);
        metric_set.upsert("test", 5.0);

        let metric = metric_set.get("test").unwrap();

        assert_eq!(metric.count(), 5);
        assert_eq!(metric.last_value(), 5.0);
        assert_eq!(metric.value_mean().unwrap(), 3.0);
        assert_eq!(metric.value_variance().unwrap(), 2.5);
        assert_eq!(metric.value_std_dev().unwrap(), 1.5811388);
        assert_eq!(metric.value_min().unwrap(), 1.0);
        assert_eq!(metric.value_max().unwrap(), 5.0);
    }

    #[test]
    fn test_metric_labels() {
        let mut metric = Metric::new("test");
        metric.add_label(MetricLabel::new("label1", "value1"));
        metric.add_label(MetricLabel::new("label2", "value2"));
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
        assert!(metric.labels().is_some());
        let labels = metric.labels().unwrap();
        assert!(labels.contains(&MetricLabel::new("label1", "value1")));
        assert!(labels.contains(&MetricLabel::new("label2", "value2")));
    }
}
