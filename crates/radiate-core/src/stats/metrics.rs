use super::Statistic;
use crate::{Distribution, TimeStatistic};
use std::{collections::BTreeMap, fmt::Debug, time::Duration};

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

    pub fn upsert_value(&mut self, name: &'static str, value: f32) {
        if let Some(m) = self.metrics.get_mut(name) {
            m.add_value(value);
        } else {
            self.add(Metric::new_value(name));
            self.upsert_value(name, value);
        }
    }

    pub fn upsert_time(&mut self, name: &'static str, value: Duration) {
        if let Some(m) = self.metrics.get_mut(name) {
            m.add_duration(value);
        } else {
            self.add(Metric::new_time(name));
            self.upsert_time(name, value);
        }
    }

    pub fn upsert_distribution(&mut self, name: &'static str, value: &[f32]) {
        if let Some(m) = self.metrics.get_mut(name) {
            m.add_distribution(value);
        } else {
            self.add(Metric::new_distribution(name));
            self.upsert_distribution(name, value);
        }
    }

    pub fn upsert_operations(
        &mut self,
        name: &'static str,
        value: impl Into<f32>,
        time: impl Into<Duration>,
    ) {
        if let Some(m) = self.metrics.get_mut(name) {
            m.add_value(value.into());
            m.add_duration(time.into());
        } else {
            self.add(Metric::new_operations(name, value.into(), time.into()));
        }
    }

    pub fn upsert(&mut self, metric: Metric) {
        if let Some(m) = self.metrics.get_mut(metric.name()) {
            match m {
                Metric::Value(_, stat, dist) => {
                    if let Metric::Value(_, new_stat, new_dist) = metric {
                        stat.add(new_stat.last_value());
                        dist.add(new_dist.last_sequence());
                    }
                }
                Metric::Time(_, stat) => {
                    if let Metric::Time(_, new_stat) = metric {
                        stat.add(new_stat.last_time());
                    }
                }
                Metric::Distribution(_, dist) => {
                    if let Metric::Distribution(_, new_dist) = metric {
                        dist.add(new_dist.last_sequence());
                    }
                }
                Metric::Operations(_, stat, time_stat) => {
                    if let Metric::Operations(_, new_stat, new_time_stat) = metric {
                        stat.add(new_stat.last_value());
                        time_stat.add(new_time_stat.last_time());
                    }
                }
            }
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
}

impl Debug for MetricSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MetricSet {{\n")?;

        for meteric in self
            .iter()
            .filter(|(_, m)| matches!(m, Metric::Operations(_, _, _)))
        {
            write!(f, "\t{:?},\n", meteric.1)?;
        }

        for metric in self
            .iter()
            .filter(|(_, m)| matches!(m, Metric::Value(_, _, _)))
        {
            write!(f, "\t{:?},\n", metric.1)?;
        }
        for metric in self
            .iter()
            .filter(|(_, m)| matches!(m, Metric::Distribution(_, _)))
        {
            write!(f, "\t{:?},\n", metric.1)?;
        }
        for metric in self.iter().filter(|(_, m)| matches!(m, Metric::Time(_, _))) {
            write!(f, "\t{:?},\n", metric.1)?;
        }

        write!(f, "}}")
    }
}

#[derive(Clone, PartialEq)]
pub enum Metric {
    Value(&'static str, Statistic, Distribution),
    Time(&'static str, TimeStatistic),
    Distribution(&'static str, Distribution),
    Operations(&'static str, Statistic, TimeStatistic),
}

impl Metric {
    pub fn new_value(name: &'static str) -> Self {
        Metric::Value(name, Statistic::default(), Distribution::default())
    }

    pub fn new_time(name: &'static str) -> Self {
        Metric::Time(name, TimeStatistic::default())
    }

    pub fn new_distribution(name: &'static str) -> Self {
        Metric::Distribution(name, Distribution::default())
    }

    pub fn new_operations(
        name: &'static str,
        val: impl Into<Statistic>,
        time: impl Into<TimeStatistic>,
    ) -> Self {
        Metric::Operations(name, val.into(), time.into())
    }

    pub fn with_value(mut self, value: impl Into<f32>) -> Self {
        match &mut self {
            Metric::Value(_, stat, dist) => {
                let into_value = value.into();
                stat.add(into_value);
                dist.push(into_value);
            }
            Metric::Operations(_, stat, _) => {
                let into_value = value.into();
                stat.add(into_value);
            }
            _ => {}
        }
        self
    }

    pub fn with_distribution(mut self, values: &[f32]) -> Self {
        match &mut self {
            Metric::Value(_, _, dist) => {
                dist.add(values);
            }
            Metric::Distribution(_, dist) => {
                dist.add(values);
            }
            _ => {}
        }
        self
    }

    pub fn with_time(mut self, value: impl Into<Duration>) -> Self {
        match &mut self {
            Metric::Time(_, stat) => {
                let into_value = value.into();
                stat.add(into_value);
            }
            Metric::Operations(_, _, stat) => {
                let into_value = value.into();
                stat.add(into_value);
            }
            _ => {}
        }
        self
    }

    pub fn with_operations(mut self, value: impl Into<f32>, time: impl Into<Duration>) -> Self {
        match &mut self {
            Metric::Operations(_, stat, time_stat) => {
                let into_value = value.into();
                stat.add(into_value);
                time_stat.add(time.into());
            }
            _ => {}
        }
        self
    }

    pub fn with_count_value(self, count: impl Into<usize>) -> Self {
        self.with_value(count.into() as f32)
    }

    pub fn add_value(&mut self, value: f32) {
        match self {
            Metric::Value(_, stat, dist) => {
                stat.add(value);
                dist.push(value);
            }
            Metric::Operations(_, stat, _) => stat.add(value),
            Metric::Distribution(_, dist) => dist.push(value),
            _ => {}
        }
    }

    pub fn add_duration(&mut self, value: Duration) {
        match self {
            Metric::Time(_, stat) => stat.add(value),
            Metric::Operations(_, _, stat) => stat.add(value),
            _ => {}
        }
    }

    pub fn add_distribution(&mut self, value: &[f32]) {
        if let Metric::Distribution(_, dist) = self {
            dist.add(value);
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Metric::Value(name, _, _) => name,
            Metric::Time(name, _) => name,
            Metric::Distribution(name, _) => name,
            Metric::Operations(name, _, _) => name,
        }
    }

    pub fn last_value(&self) -> f32 {
        match self {
            Metric::Value(_, stat, _) => stat.last_value(),
            Metric::Operations(_, stat, _) => stat.last_value(),
            _ => 0.0,
        }
    }

    pub fn last_time(&self) -> Duration {
        match self {
            Metric::Time(_, stat) => stat.last_time(),
            Metric::Operations(_, _, stat) => stat.last_time(),
            _ => Duration::from_secs_f32(0.0),
        }
    }

    pub fn value_mean(&self) -> Option<f32> {
        match self {
            Metric::Value(_, stat, _) => Some(stat.mean()),
            Metric::Operations(_, stat, _) => Some(stat.mean()),
            _ => None,
        }
    }

    pub fn value_variance(&self) -> Option<f32> {
        match self {
            Metric::Value(_, stat, _) => Some(stat.variance()),
            Metric::Operations(_, stat, _) => Some(stat.variance()),
            _ => None,
        }
    }

    pub fn value_std_dev(&self) -> Option<f32> {
        match self {
            Metric::Value(_, stat, _) => Some(stat.std_dev()),
            Metric::Operations(_, stat, _) => Some(stat.std_dev()),
            _ => None,
        }
    }

    pub fn value_skewness(&self) -> Option<f32> {
        match self {
            Metric::Value(_, stat, _) => Some(stat.skewness()),
            Metric::Operations(_, stat, _) => Some(stat.skewness()),
            _ => None,
        }
    }

    pub fn value_min(&self) -> Option<f32> {
        match self {
            Metric::Value(_, stat, _) => Some(stat.min()),
            Metric::Operations(_, stat, _) => Some(stat.min()),
            _ => None,
        }
    }

    pub fn value_max(&self) -> Option<f32> {
        match self {
            Metric::Value(_, stat, _) => Some(stat.max()),
            Metric::Operations(_, stat, _) => Some(stat.max()),
            _ => None,
        }
    }

    pub fn time_mean(&self) -> Option<Duration> {
        match self {
            Metric::Time(_, stat) => Some(stat.mean()),
            Metric::Operations(_, _, stat) => Some(stat.mean()),
            _ => None,
        }
    }

    pub fn time_variance(&self) -> Option<Duration> {
        match self {
            Metric::Time(_, stat) => Some(stat.variance()),
            Metric::Operations(_, _, stat) => Some(stat.variance()),
            _ => None,
        }
    }

    pub fn time_std_dev(&self) -> Option<Duration> {
        match self {
            Metric::Time(_, stat) => Some(stat.standard_deviation()),
            Metric::Operations(_, _, stat) => Some(stat.standard_deviation()),
            _ => None,
        }
    }

    pub fn time_min(&self) -> Option<Duration> {
        match self {
            Metric::Time(_, stat) => Some(stat.min()),
            Metric::Operations(_, _, stat) => Some(stat.min()),
            _ => None,
        }
    }

    pub fn time_max(&self) -> Option<Duration> {
        match self {
            Metric::Time(_, stat) => Some(stat.max()),
            Metric::Operations(_, _, stat) => Some(stat.max()),
            _ => None,
        }
    }

    pub fn time_sum(&self) -> Option<Duration> {
        match self {
            Metric::Time(_, stat) => Some(stat.sum()),
            Metric::Operations(_, _, stat) => Some(stat.sum()),
            _ => None,
        }
    }

    pub fn last_sequence(&self) -> Option<&Vec<f32>> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.last_sequence()),
            Metric::Value(_, _, dist) => Some(dist.last_sequence()),
            _ => None,
        }
    }

    pub fn distribution_mean(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.mean()),
            Metric::Value(_, _, dist) => Some(dist.mean()),
            _ => None,
        }
    }

    pub fn distribution_variance(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.variance()),
            Metric::Value(_, _, dist) => Some(dist.variance()),
            _ => None,
        }
    }

    pub fn distribution_std_dev(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.standard_deviation()),
            Metric::Value(_, _, dist) => Some(dist.standard_deviation()),
            _ => None,
        }
    }

    pub fn distribution_skewness(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.skewness()),
            Metric::Value(_, _, dist) => Some(dist.skewness()),
            _ => None,
        }
    }

    pub fn distribution_kurtosis(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.kurtosis()),
            Metric::Value(_, _, dist) => Some(dist.kurtosis()),
            _ => None,
        }
    }

    pub fn distribution_min(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.min()),
            Metric::Value(_, _, dist) => Some(dist.min()),
            _ => None,
        }
    }

    pub fn distribution_max(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.max()),
            Metric::Value(_, _, dist) => Some(dist.max()),
            _ => None,
        }
    }

    pub fn count(&self) -> i32 {
        match self {
            Metric::Value(_, stat, _) => stat.count(),
            Metric::Time(_, stat) => stat.count(),
            Metric::Distribution(_, dist) => dist.count(),
            Metric::Operations(_, stat, _) => stat.count(),
        }
    }
}

impl std::fmt::Debug for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Metric::Value(name, stat, dist) => write!(
                f,
                "{:<20} | Mean: {:>8.3}, Min: {:>8.3}, Max: {:>8.3}, N: {:>3} | Dist. Mean: {:>8.3}, Dist. StdDev: {:>8.3}, Dist. Min: {:>8.3}, Dist. Max: {:>8.3}",
                name,
                stat.mean(),
                stat.min(),
                stat.max(),
                stat.count(),
                dist.mean(),
                dist.standard_deviation(),
                dist.min(),
                dist.max(),
            ),
            Metric::Time(name, stat) => write!(
                f,
                "{:<20} | Avg Time: {:>9.3?}, Min Time: {:>9.3?}, Max Time: {:>9.3?}, N: {:>3} | Total Time: {:>9.3?}",
                name,
                stat.mean(),
                stat.min(),
                stat.max(),
                stat.count(),
                stat.sum(),
            ),
            Metric::Distribution(name, dist) => write!(
                f,
                "{:<20} | Mean: {:>8.3}, StdDev: {:>8.3}, Min: {:>8.3}, Max: {:>8.3}, N: {:>3}",
                name,
                dist.mean(),
                dist.standard_deviation(),
                dist.min(),
                dist.max(),
                dist.count(),
            ),
            Metric::Operations(name, stat, time_stat) => write!(
                f,
                "{:<20} | Mean: {:>8.3}, Min: {:>8.3}, Max: {:>8.3}, N: {:>3} | Avg Time: {:>9.3?}, Total Time: {:>9.3?}",
                name,
                stat.mean(),
                stat.min(),
                stat.max(),
                stat.count(),
                time_stat.mean(),
                time_stat.sum(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric() {
        let mut metric = Metric::Value("test", Statistic::default(), Distribution::default());
        metric.add_value(1.0);
        metric.add_value(2.0);
        metric.add_value(3.0);
        metric.add_value(4.0);
        metric.add_value(5.0);

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
        metric_set.upsert_value("test", 1.0);
        metric_set.upsert_value("test", 2.0);
        metric_set.upsert_value("test", 3.0);
        metric_set.upsert_value("test", 4.0);
        metric_set.upsert_value("test", 5.0);

        let metric = metric_set.get("test").unwrap();

        assert_eq!(metric.count(), 5);
        assert_eq!(metric.last_value(), 5.0);
        assert_eq!(metric.value_mean().unwrap(), 3.0);
        assert_eq!(metric.value_variance().unwrap(), 2.5);
        assert_eq!(metric.value_std_dev().unwrap(), 1.5811388);
        assert_eq!(metric.value_min().unwrap(), 1.0);
        assert_eq!(metric.value_max().unwrap(), 5.0);
    }
}
