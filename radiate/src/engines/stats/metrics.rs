use super::Statistic;
use crate::{Distribution, TimeStatistic};
use std::{collections::BTreeMap, time::Duration};

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

    pub fn upsert_sequence(&mut self, name: &'static str, value: &[f32]) {
        if let Some(m) = self.metrics.get_mut(name) {
            m.add_sequence(value);
        } else {
            self.add(Metric::new_distribution(name));
            self.upsert_sequence(name, value);
        }
    }

    pub fn upsert_operations(&mut self, name: &'static str, value: f32, time: Duration) {
        if let Some(m) = self.metrics.get_mut(name) {
            m.add_value(value);
            m.add_duration(time);
        } else {
            self.add(Metric::new_operations(name, value, time));
        }
    }

    pub fn upsert(&mut self, metric: Metric) {
        if let Some(m) = self.metrics.get_mut(metric.name()) {
            match m {
                Metric::Value(_, stat) => {
                    if let Metric::Value(_, new_stat) = metric {
                        stat.add(new_stat.last_value());
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

#[derive(Clone, PartialEq)]
pub enum Metric {
    Value(&'static str, Statistic),
    Time(&'static str, TimeStatistic),
    Distribution(&'static str, Distribution),
    Operations(&'static str, Statistic, TimeStatistic),
}

impl Metric {
    pub fn new_value(name: &'static str) -> Self {
        Metric::Value(name, Statistic::default())
    }

    pub fn new_time(name: &'static str) -> Self {
        Metric::Time(name, TimeStatistic::default())
    }

    pub fn new_distribution(name: &'static str) -> Self {
        Metric::Distribution(name, Distribution::default())
    }

    pub fn new_operations(name: &'static str, val: f32, time: Duration) -> Self {
        Metric::Operations(name, Statistic::new(val), TimeStatistic::new(time))
    }

    pub fn add_value(&mut self, value: f32) {
        match self {
            Metric::Value(_, stat) => stat.add(value),
            Metric::Operations(_, stat, _) => stat.add(value),
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

    pub fn add_sequence(&mut self, value: &[f32]) {
        if let Metric::Distribution(_, dist) = self {
            dist.add(value);
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Metric::Value(name, _) => name,
            Metric::Time(name, _) => name,
            Metric::Distribution(name, _) => name,
            Metric::Operations(name, _, _) => name,
        }
    }

    pub fn last_value(&self) -> f32 {
        match self {
            Metric::Value(_, stat) => stat.last_value(),
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
            Metric::Value(_, stat) => Some(stat.mean()),
            Metric::Operations(_, stat, _) => Some(stat.mean()),
            _ => None,
        }
    }

    pub fn value_variance(&self) -> Option<f32> {
        match self {
            Metric::Value(_, stat) => Some(stat.variance()),
            Metric::Operations(_, stat, _) => Some(stat.variance()),
            _ => None,
        }
    }

    pub fn value_std_dev(&self) -> Option<f32> {
        match self {
            Metric::Value(_, stat) => Some(stat.std_dev()),
            Metric::Operations(_, stat, _) => Some(stat.std_dev()),
            _ => None,
        }
    }

    pub fn value_skewness(&self) -> Option<f32> {
        match self {
            Metric::Value(_, stat) => Some(stat.skewness()),
            Metric::Operations(_, stat, _) => Some(stat.skewness()),
            _ => None,
        }
    }

    pub fn value_min(&self) -> Option<f32> {
        match self {
            Metric::Value(_, stat) => Some(stat.min()),
            Metric::Operations(_, stat, _) => Some(stat.min()),
            _ => None,
        }
    }

    pub fn value_max(&self) -> Option<f32> {
        match self {
            Metric::Value(_, stat) => Some(stat.max()),
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
            _ => None,
        }
    }

    pub fn sequence_mean(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.mean()),
            _ => None,
        }
    }

    pub fn sequence_variance(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.variance()),
            _ => None,
        }
    }

    pub fn sequence_std_dev(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.standard_deviation()),
            _ => None,
        }
    }

    pub fn sequence_skewness(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.skewness()),
            _ => None,
        }
    }

    pub fn sequence_kurtosis(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.kurtosis()),
            _ => None,
        }
    }

    pub fn sequence_min(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.min()),
            _ => None,
        }
    }

    pub fn sequence_max(&self) -> Option<f32> {
        match self {
            Metric::Distribution(_, dist) => Some(dist.max()),
            _ => None,
        }
    }

    pub fn count(&self) -> i32 {
        match self {
            Metric::Value(_, stat) => stat.count(),
            Metric::Time(_, stat) => stat.count(),
            Metric::Distribution(_, dist) => dist.count(),
            Metric::Operations(_, stat, _) => stat.count(),
        }
    }
}

impl std::fmt::Debug for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Metric::Value(name, stat) => write!(
                f,
                "Metric Value {{ {:<15} -> ∧: {:<7.3?}  ∨: {:<7.3?} μ: {:<7.3?} N: {:<7} }}",
                name,
                stat.max(),
                stat.min(),
                stat.mean(),
                stat.count()
            ),
            Metric::Time(name, stat) => write!(
                f,
                "Metric Time {{ {:<15} -> ∧: {:<7.3?}  ∨: {:<7.3?} μ: {:<7.3?} N: {:<7} S: {:<7.3?} }}",
                name,
                stat.max(),
                stat.min(),
                stat.mean(),
                stat.count(),
                stat.sum()
            ),
            Metric::Distribution(name, dist) => write!(
                f,
                "Metric Dist. {{ {:<15} -> ∧: {:<7.3?}  ∨: {:<7.3?} μ: {:<7.3?} N: {:<7} }}",
                name,
                dist.max(),
                dist.min(),
                dist.mean(),
                dist.count()
            ),
            Metric::Operations(name, stat, time_stat) => write!(
                f,
                "Metric Oper. {{ {:<15} -> ∧: {:<7.3?}  ∨: {:<7.3?} μ: {:<7.3?} N: {:<7} :: ∧[{:<10?}] ∨[{:<10?}] μ[{:<10?}] S[{:<10.3?}] }}",
                name,
                stat.max(),
                stat.min(),
                stat.mean(),
                stat.count(),
                time_stat.max(),
                time_stat.min(),
                time_stat.mean(),
                time_stat.sum()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metric() {
        let mut metric = Metric::Value("test", Statistic::default());
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
}
