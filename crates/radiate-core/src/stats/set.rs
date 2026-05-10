use crate::{
    Metric, MetricUpdate,
    stats::{
        Meta, Tag, TagType,
        defaults::try_add_tag_from_str,
        expression::{ExprProjection, SelectExpr},
        fmt,
    },
};
use radiate_utils::{AnyValue, DataType, SmallStr};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
    time::Duration,
};

#[derive(PartialEq)]
pub struct MetricSetSummary {
    pub metrics: usize,
    pub updates: f32,
}

#[derive(Clone, Default, PartialEq)]
pub struct MetricSet {
    metrics: HashMap<SmallStr, Metric>,
    meta: Meta,
}

impl MetricSet {
    pub fn new() -> Self {
        MetricSet {
            metrics: HashMap::new(),
            meta: Meta::default(),
        }
    }

    pub fn next_version(&mut self) -> u64 {
        let result = self.meta.version;
        self.meta.version += 1;
        result
    }

    pub fn version(&self) -> u64 {
        self.meta.version
    }

    #[inline(always)]
    pub fn keys(&self) -> Vec<SmallStr> {
        self.metrics.keys().cloned().collect()
    }

    #[inline(always)]
    pub fn flush_all_into(&mut self, target: &mut MetricSet) {
        let version = target.next_version();
        for (key, mut m) in self.metrics.drain() {
            m.set_version(version);
            if let Some(target_metric) = target.metrics.get_mut(key.as_str()) {
                target_metric.update_from(m);
            } else {
                try_add_tag_from_str(&mut m);
                target.metrics.insert(key, m);
            }
        }

        self.clear();
    }

    #[inline(always)]
    pub fn replace(&mut self, metric: impl Into<Metric>) {
        let mut metric = metric.into();
        try_add_tag_from_str(&mut metric);
        self.metrics.insert(metric.name().clone(), metric);
    }

    #[inline(always)]
    pub fn upsert<'a>(&mut self, metric: impl Into<MetricSetUpdate<'a>>) {
        let update = metric.into();
        let version = self.version();

        match update {
            MetricSetUpdate::Many(metrics) => {
                for metric in metrics {
                    self.add_or_update_internal(version, metric);
                }
            }
            MetricSetUpdate::Single(metric) => {
                self.add_or_update_internal(version, metric);
            }
            MetricSetUpdate::ManyUpdate(updates) => {
                for metric in updates {
                    self.upsert(metric);
                }
            }
            MetricSetUpdate::NamedSingle(name, metric_update, tag) => {
                self.meta.update_count += 1;
                if let Some(m) = self.metrics.get_mut(name) {
                    m.set_version(version);
                    m.apply_update(metric_update);
                    if let Some(tag) = tag {
                        m.add_tag(tag);
                    }
                    return;
                }

                let new_name = radiate_utils::intern_name_as_snake_case(name);
                if let Some(m) = self.metrics.get_mut(new_name) {
                    m.set_version(version);
                    m.apply_update(metric_update);
                    if let Some(tag) = tag {
                        m.add_tag(tag);
                    }
                } else {
                    let mut metric = Metric::new(new_name);
                    try_add_tag_from_str(&mut metric);
                    metric.set_version(version);
                    metric.apply_update(metric_update);

                    if let Some(tag) = tag {
                        metric.add_tag(tag);
                    }

                    self.add(metric);
                }
            }
        }
    }

    #[inline(always)]
    pub fn iter_tagged(&self, tag: TagType) -> impl Iterator<Item = (&str, &Metric)> {
        self.metrics.iter().filter_map(move |(k, m)| {
            if m.tags().has(tag) {
                Some((k.as_str(), m))
            } else {
                None
            }
        })
    }

    #[inline(always)]
    pub fn tags(&self) -> impl Iterator<Item = TagType> {
        self.metrics
            .values()
            .fold(Tag::empty(), |acc, m| acc.union(m.tags()))
            .into_iter()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Metric)> {
        self.metrics
            .iter()
            .map(|(name, metric)| (name.as_str(), metric))
    }

    #[inline(always)]
    pub fn add(&mut self, metric: Metric) {
        self.metrics.insert(metric.name().clone(), metric);
    }

    #[inline(always)]
    pub fn get(&self, name: &str) -> Option<&Metric> {
        self.metrics.get(name)
    }

    #[inline(always)]
    pub fn get_from_string(&self, name: String) -> Option<&Metric> {
        self.metrics.get(name.as_str())
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        for (_, m) in self.metrics.iter_mut() {
            m.clear_values();
        }

        self.meta.update_count = 0;
    }

    #[inline(always)]
    pub fn contains_key(&self, name: &str) -> bool {
        self.metrics.contains_key(name)
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }

    #[inline(always)]
    pub fn summary(&self) -> MetricSetSummary {
        MetricSetSummary {
            metrics: self.metrics.len(),
            updates: self.meta.update_count as f32,
        }
    }

    pub fn dashboard(&self) -> String {
        fmt::render_full(self).unwrap_or_default()
    }

    fn add_or_update_internal(&mut self, version: u64, mut metric: Metric) {
        self.meta.update_count += 1;
        if let Some(existing) = self.metrics.get_mut(metric.name().as_str()) {
            existing.set_version(version);
            existing.update_from(metric);
        } else {
            try_add_tag_from_str(&mut metric);
            metric.set_version(version);
            self.metrics.insert(metric.name().clone(), metric);
        }
    }
}

impl ExprProjection for MetricSet {
    fn project(&self, path: &SelectExpr) -> Option<AnyValue<'static>> {
        let value_to_float32 = |value: f32| AnyValue::Float32(value);
        let value_to_duration = |value: f32| Duration::from_secs_f32(value).into();

        let SelectExpr::Field(key, field) = path else {
            return None;
        };

        let str_key = key.as_str()?;

        self.get(str_key)
            .map(|metric| match field.dtype() {
                DataType::Float32 => match field.name().to_lowercase().as_str() {
                    "last_value" => AnyValue::Float32(metric.last_value()),
                    "mean" => value_to_float32(metric.mean()),
                    "std_dev" => value_to_float32(metric.stddev()),
                    "min" => value_to_float32(metric.min()),
                    "max" => value_to_float32(metric.max()),
                    "sum" => value_to_float32(metric.sum()),
                    "skew" => value_to_float32(metric.skew()),
                    "var" => value_to_float32(metric.var()),
                    "count" => AnyValue::UInt64(metric.count() as u64),
                    "version" => AnyValue::UInt64(metric.version()),
                    "update_count" => AnyValue::UInt64(metric.update_count() as u64),
                    _ => AnyValue::Null,
                },
                DataType::Duration => match field.name().to_lowercase().as_str() {
                    "last_value" => {
                        AnyValue::Duration(Duration::from_secs_f32(metric.last_value()))
                    }
                    "mean" => value_to_duration(metric.mean()),
                    "std_dev" => value_to_duration(metric.stddev()),
                    "min" => value_to_duration(metric.min()),
                    "max" => value_to_duration(metric.max()),
                    "sum" => value_to_duration(metric.sum()),
                    "var" => value_to_duration(metric.var()),
                    "count" => AnyValue::UInt64(metric.count() as u64),
                    "version" => AnyValue::UInt64(metric.version()),
                    "update_count" => AnyValue::UInt64(metric.update_count() as u64),
                    _ => AnyValue::Null,
                },
                _ => AnyValue::Null,
            })
            .or_else(|| Some(AnyValue::Null))
    }
}

impl Display for MetricSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = self.summary();
        let out = format!(
            "[{} metrics, {:.0} updates]",
            summary.metrics, summary.updates
        );
        write!(f, "{out}\n{}", fmt::render_full(self).unwrap_or_default())?;
        Ok(())
    }
}

impl Debug for MetricSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "MetricSet {{")?;
        writeln!(f, "{}", fmt::render_dashboard(self).unwrap_or_default())?;
        write!(f, "}}")
    }
}

#[cfg(feature = "serde")]
impl Serialize for MetricSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let metrics = self.metrics.values().cloned().collect::<Vec<Metric>>();
        metrics.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for MetricSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let metrics = Vec::<Metric>::deserialize(deserializer)?;

        let mut metric_set = MetricSet::new();
        for metric in metrics {
            metric_set.add(metric);
        }

        Ok(metric_set)
    }
}

#[derive(Debug)]
pub enum MetricSetUpdate<'a> {
    Many(Vec<Metric>),
    Single(Metric),
    ManyUpdate(Vec<(&'static str, MetricUpdate<'a>)>),
    NamedSingle(&'static str, MetricUpdate<'a>, Option<TagType>),
}

impl From<Vec<Metric>> for MetricSetUpdate<'_> {
    fn from(metrics: Vec<Metric>) -> Self {
        MetricSetUpdate::Many(metrics)
    }
}

impl From<Metric> for MetricSetUpdate<'_> {
    fn from(metric: Metric) -> Self {
        MetricSetUpdate::Single(metric)
    }
}

impl<'a, U> From<(&'static str, U)> for MetricSetUpdate<'a>
where
    U: Into<MetricUpdate<'a>>,
{
    fn from((name, update): (&'static str, U)) -> Self {
        MetricSetUpdate::NamedSingle(name, update.into(), None)
    }
}

impl<'a, U> From<(TagType, &'static str, U)> for MetricSetUpdate<'a>
where
    U: Into<MetricUpdate<'a>>,
{
    fn from((tag, name, update): (TagType, &'static str, U)) -> Self {
        MetricSetUpdate::NamedSingle(name, update.into(), Some(tag))
    }
}

impl<'a, U> From<(&'static str, U, usize)> for MetricSetUpdate<'a>
where
    U: Into<MetricUpdate<'a>>,
{
    fn from((name, update, count): (&'static str, U, usize)) -> Self {
        let name = radiate_utils::intern!(format!("{name}.{count}"));
        MetricSetUpdate::NamedSingle(name, update.into(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }

    fn assert_stat_eq(m: &Metric, count: u32, mean: f32, var: f32, min: f32, max: f32) {
        assert_eq!(m.count(), count);
        assert!(approx_eq(m.mean(), mean, EPSILON), "mean");
        assert!(approx_eq(m.var(), var, EPSILON), "var");
        assert!(approx_eq(m.min(), min, EPSILON), "min");
        assert!(approx_eq(m.max(), max, EPSILON), "max");
    }

    fn stats_of(values: &[f32]) -> (u32, f32, f32, f32, f32) {
        // sample variance (n-1), matches your Statistic::variance
        let n = values.len() as u32;
        if n == 0 {
            return (0, 0.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY);
        }
        let mean = values.iter().sum::<f32>() / values.len() as f32;

        let mut m2 = 0.0_f32;
        for &v in values {
            let d = v - mean;
            m2 += d * d;
        }

        let var = if n == 1 { 0.0 } else { m2 / (n as f32 - 1.0) };

        let min = values.iter().cloned().fold(f32::INFINITY, f32::min);
        let max = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);

        (n, mean, var, min, max)
    }

    #[test]
    fn metric_set_flush_all_into_merges_metrics() {
        let mut a = MetricSet::new();
        let mut b = MetricSet::new();

        a.upsert(("scores", &[1.0, 2.0, 3.0][..]));
        b.upsert(("scores", &[10.0, 20.0][..]));

        // move a into b
        a.flush_all_into(&mut b);

        let m = b.get("scores").unwrap();
        let combined = [1.0, 2.0, 3.0, 10.0, 20.0];
        let (n, mean, var, min, max) = stats_of(&combined);
        assert_stat_eq(m, n, mean, var, min, max);
    }
}
