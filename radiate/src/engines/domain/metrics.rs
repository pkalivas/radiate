use std::{collections::BTreeMap, time::Duration};

use super::Statistic;

pub const METRIC_SCORE: &str = "score";
pub const METRIC_AGE: &str = "age";
pub const METRIC_EVALUATE: &str = "evaluate";
pub const METRIC_AGE_FILTER: &str = "age_filter";
pub const METRIC_INVALID_FILTER: &str = "invalid_filter";
pub const METRIC_UNIQUE: &str = "unique";

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

    pub fn upsert(&mut self, name: &'static str, value: f32, time: Duration) {
        if let Some(metric) = self.metrics.get_mut(name) {
            metric.add(value, time);
        } else {
            self.add(Metric::new(name));
            self.metrics.get_mut(name).unwrap().add(value, time);
        }
    }

    pub fn upsert_metric(&mut self, metric: Metric) {
        if let Some(m) = self.metrics.get_mut(metric.name()) {
            m.add_value(metric.last_value());
            m.add_time(metric.last_time());
        } else {
            self.metrics.insert(metric.name(), metric);
        }
    }

    pub fn upsert_value(&mut self, name: &'static str, value: f32) {
        if let Some(metric) = self.metrics.get_mut(name) {
            metric.add_value(value);
        } else {
            self.add(Metric::new(name));
            self.metrics.get_mut(name).unwrap().add_value(value);
        }
    }

    pub fn upsert_time(&mut self, name: &'static str, value: Duration) {
        if let Some(metric) = self.metrics.get_mut(name) {
            metric.add_time(value);
        } else {
            self.add(Metric::new(name));
            self.metrics.get_mut(name).unwrap().add_time(value);
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
}

impl std::fmt::Debug for MetricSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MetricSet {{\n")?;
        for name in self.names() {
            write!(f, "  \t{:?},\n", self.get(name).unwrap())?;
        }
        write!(f, "}}")
    }
}

#[derive(Clone)]
pub struct Metric {
    pub name: &'static str,
    pub stats: Statistic,
    pub time_stats: Statistic,
}

impl Metric {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            stats: Statistic::new(),
            time_stats: Statistic::new(),
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn add(&mut self, value: f32, time: Duration) {
        self.add_value(value);
        self.add_time(time);
    }

    pub fn add_value(&mut self, value: f32) {
        self.stats.add(value);
    }

    pub fn add_time(&mut self, value: Duration) {
        self.time_stats.add(value.as_secs_f32());
    }

    pub fn last_value(&self) -> f32 {
        self.stats.last_value()
    }

    pub fn variance(&self) -> f32 {
        self.stats.variance()
    }

    pub fn skewness(&self) -> f32 {
        self.stats.skewness()
    }

    pub fn std_dev(&self) -> f32 {
        self.stats.std_dev()
    }

    pub fn mean(&self) -> f32 {
        self.stats.mean()
    }

    pub fn min(&self) -> f32 {
        self.stats.min()
    }

    pub fn max(&self) -> f32 {
        self.stats.max()
    }

    pub fn count(&self) -> i32 {
        self.stats.count()
    }

    pub fn last_time(&self) -> Duration {
        Duration::from_secs_f32(self.time_stats.last_value())
    }

    pub fn mean_time(&self) -> Duration {
        if self.time_stats.count() == 0 {
            return Duration::from_secs_f32(0.0);
        }

        Duration::from_secs_f32(self.time_stats.mean())
    }

    pub fn min_time(&self) -> Duration {
        if self.time_stats.count() == 0 {
            return Duration::from_secs_f32(0.0);
        }

        Duration::from_secs_f32(self.time_stats.min())
    }

    pub fn max_time(&self) -> Duration {
        if self.time_stats.count() == 0 {
            return Duration::from_secs_f32(0.0);
        }

        Duration::from_secs_f32(self.time_stats.max())
    }
}

impl std::fmt::Debug for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Metric {{ {:<15} -> ∧: {:<7.3?}  ∨: {:<7.3?} μ: {:<7.3?} N: {:<7} :: ∧[{:<10?}] ∨[{:<10?}] μ[{:<10?}] }}",
            self.name(),
            self.max(),
            self.min(),
            self.mean(),
            self.count(),
            self.max_time(),
            self.min_time(),
            self.mean_time()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric() {
        let mut metric = Metric::new("test");
        metric.add_value(1.0);
        metric.add_value(2.0);
        metric.add_value(3.0);
        metric.add_value(4.0);
        metric.add_value(5.0);

        assert_eq!(metric.count(), 5);
        assert_eq!(metric.last_value(), 5.0);
        assert_eq!(metric.mean(), 3.0);
        assert_eq!(metric.variance(), 2.5);
        assert_eq!(metric.std_dev(), 1.5811388);
        assert_eq!(metric.skewness(), 0.0);
        assert_eq!(metric.min(), 1.0);
        assert_eq!(metric.max(), 5.0);
        assert_eq!(metric.name(), "test");
    }
}
