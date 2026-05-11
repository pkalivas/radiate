use crate::{
    Metric, MetricUpdate,
    stats::{
        Meta, Tag, TagType,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct MetricIdx(u32);

impl MetricIdx {
    pub const INVALID: MetricIdx = MetricIdx(u32::MAX);

    #[inline(always)]
    pub const fn new(idx: u32) -> Self {
        MetricIdx(idx)
    }

    #[inline(always)]
    pub const fn get(self) -> u32 {
        self.0
    }

    #[inline(always)]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    #[inline(always)]
    pub const fn is_valid(self) -> bool {
        self.0 != u32::MAX
    }
}

impl Default for MetricIdx {
    #[inline(always)]
    fn default() -> Self {
        MetricIdx::INVALID
    }
}

#[derive(PartialEq)]
pub struct MetricSetSummary {
    pub metrics: usize,
    pub updates: f32,
}

#[derive(Clone, Default, PartialEq)]
pub struct MetricSet {
    metrics: Vec<Metric>,
    name_lookup: HashMap<SmallStr, MetricIdx>,
    meta: Meta,
}

impl MetricSet {
    pub fn new() -> Self {
        MetricSet {
            metrics: Vec::new(),
            name_lookup: HashMap::new(),
            meta: Meta::default(),
        }
    }

    pub fn advance_generation(&mut self) -> u64 {
        let result = self.meta.generation;
        self.meta.generation += 1;
        result
    }

    pub fn generation(&self) -> u64 {
        self.meta.generation
    }

    /// Resolve a name to a stable [`MetricIdx`], registering an empty metric if
    /// the name has not been seen before. The returned handle is valid for the
    /// lifetime of this `MetricSet`.
    pub fn resolve(&mut self, name: &SmallStr) -> MetricIdx {
        if let Some(&idx) = self.name_lookup.get(name.as_str()) {
            return idx;
        }

        let idx = MetricIdx::new(self.metrics.len() as u32);
        self.name_lookup.insert(name.clone(), idx);
        self.metrics.push(Metric::new(name.clone()));
        idx
    }

    #[inline]
    pub fn get_idx(&self, name: impl AsRef<str>) -> Option<MetricIdx> {
        self.name_lookup.get(name.as_ref()).copied()
    }

    #[inline]
    pub fn upsert_at<'a>(&mut self, idx: MetricIdx, update: impl Into<MetricUpdate<'a>>) {
        let generation = self.meta.generation;
        let mmetric = &mut self.metrics[idx.as_usize()];

        mmetric.set_generation(generation);
        mmetric.apply_update(update.into());

        self.meta.update_count += 1;
    }

    #[inline(always)]
    pub fn upsert<'a>(&mut self, metric: impl Into<MetricSetUpdate<'a>>) {
        let update = metric.into();

        match update {
            MetricSetUpdate::Slot(idx, metric_update) => {
                self.upsert_at(idx, metric_update);
            }
            MetricSetUpdate::NamedSingle(name, metric_update, tag) => {
                let idx = self.resolve(&name);
                self.upsert_at(idx, metric_update);
                if let Some(tag) = tag {
                    self.metrics[idx.as_usize()].add_tag(tag);
                }
            }
        }
    }

    /// Read by handle.
    #[inline]
    pub fn get_by_idx(&self, idx: MetricIdx) -> Option<&Metric> {
        self.metrics.get(idx.as_usize())
    }

    #[inline(always)]
    pub fn keys(&self) -> impl Iterator<Item = SmallStr> {
        self.metrics.iter().map(|m| m.name().clone())
    }

    #[inline(always)]
    pub fn flush_all_into(&mut self, target: &mut MetricSet) {
        let generation = target.advance_generation();
        for mut m in self.metrics.drain(..) {
            m.set_generation(generation);
            if let Some(&idx) = target.name_lookup.get(m.name().as_str()) {
                target.metrics[idx.as_usize()].update_from(m);
            } else {
                let idx = MetricIdx::new(target.metrics.len() as u32);
                target.name_lookup.insert(m.name().clone(), idx);
                target.metrics.push(m);
            }
        }

        self.name_lookup.clear();
        self.meta.update_count = 0;
    }

    /// Insert or fully overwrite a metric by name. If a metric with the same
    /// name already exists, its slot is reused (handle stays valid).
    #[inline(always)]
    pub fn replace(&mut self, metric: impl Into<Metric>) {
        let metric = metric.into();
        if let Some(&idx) = self.name_lookup.get(metric.name().as_str()) {
            self.metrics[idx.as_usize()] = metric;
        } else {
            let idx = MetricIdx::new(self.metrics.len() as u32);
            self.name_lookup.insert(metric.name().clone(), idx);
            self.metrics.push(metric);
        }
    }

    #[inline(always)]
    pub fn iter_tagged(&self, tag: TagType) -> impl Iterator<Item = (&str, &Metric)> {
        self.metrics.iter().filter_map(move |m| {
            if m.tags().has(tag) {
                Some((m.name().as_str(), m))
            } else {
                None
            }
        })
    }

    #[inline(always)]
    pub fn tags(&self) -> impl Iterator<Item = TagType> {
        self.metrics
            .iter()
            .fold(Tag::empty(), |acc, m| acc.union(m.tags()))
            .into_iter()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Metric)> {
        self.metrics.iter().map(|m| (m.name().as_str(), m))
    }

    #[inline(always)]
    pub fn add(&mut self, metric: Metric) {
        self.replace(metric);
    }

    #[inline(always)]
    pub fn get(&self, name: impl AsRef<str>) -> Option<&Metric> {
        self.name_lookup
            .get(name.as_ref())
            .and_then(|idx| self.metrics.get(idx.as_usize()))
    }

    #[inline(always)]
    pub fn get_from_string(&self, name: String) -> Option<&Metric> {
        self.get(name.as_str())
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        for m in &mut self.metrics {
            m.clear_values();
        }
        self.meta.update_count = 0;
    }

    #[inline(always)]
    pub fn contains_key(&self, name: impl AsRef<str>) -> bool {
        self.name_lookup.contains_key(name.as_ref())
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
                    "generation" => AnyValue::UInt64(metric.generation()),
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
                    "generation" => AnyValue::UInt64(metric.generation()),
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
        self.metrics.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for MetricSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let metrics = Vec::<Metric>::deserialize(deserializer)?;
        let mut by_name = HashMap::with_capacity(metrics.len());
        for (i, m) in metrics.iter().enumerate() {
            by_name.insert(m.name().clone(), MetricIdx::new(i as u32));
        }
        Ok(MetricSet {
            metrics,
            name_lookup: by_name,
            meta: Meta::default(),
        })
    }
}

#[derive(Debug)]
pub enum MetricSetUpdate<'a> {
    NamedSingle(SmallStr, MetricUpdate<'a>, Option<TagType>),
    Slot(MetricIdx, MetricUpdate<'a>),
}

impl<'a> From<(MetricIdx, MetricUpdate<'a>)> for MetricSetUpdate<'a> {
    fn from((idx, update): (MetricIdx, MetricUpdate<'a>)) -> Self {
        MetricSetUpdate::Slot(idx, update)
    }
}

impl<'a, N, U> From<(N, U)> for MetricSetUpdate<'a>
where
    N: Into<SmallStr>,
    U: Into<MetricUpdate<'a>>,
{
    fn from((name, update): (N, U)) -> Self {
        MetricSetUpdate::NamedSingle(name.into(), update.into(), None)
    }
}

impl<'a, N, U> From<(TagType, N, U)> for MetricSetUpdate<'a>
where
    N: Into<SmallStr>,
    U: Into<MetricUpdate<'a>>,
{
    fn from((tag, name, update): (TagType, N, U)) -> Self {
        MetricSetUpdate::NamedSingle(name.into(), update.into(), Some(tag))
    }
}

impl<'a, N, U> From<(N, U, usize)> for MetricSetUpdate<'a>
where
    N: AsRef<str>,
    U: Into<MetricUpdate<'a>>,
{
    fn from((name, update, count): (N, U, usize)) -> Self {
        let name: SmallStr = format!("{}.{}", name.as_ref(), count).into();
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

    #[test]
    fn resolve_returns_stable_handle() {
        let mut set = MetricSet::new();
        let name = SmallStr::from_static("test.metric");

        let idx1 = set.resolve(&name);
        let idx2 = set.resolve(&name);
        assert_eq!(idx1, idx2);

        set.upsert_at(idx1, 1.0);
        set.upsert_at(idx1, 2.0);
        set.upsert_at(idx1, 3.0);

        let m = set.get(name.as_str()).unwrap();
        assert_eq!(m.count(), 3);
        assert_eq!(m.sum(), 6.0);
    }

    #[test]
    fn upsert_by_name_and_at_share_storage() {
        let mut set = MetricSet::new();
        let name: SmallStr = SmallStr::from_static("shared");

        set.upsert((name.clone(), 1.0));
        let idx = set.get_idx(name.as_str()).unwrap();
        set.upsert_at(idx, 2.0);

        let m = set.get(name.as_str()).unwrap();
        assert_eq!(m.count(), 2);
        assert_eq!(m.sum(), 3.0);
    }

    #[test]
    fn resolve_assigns_sequential_indices() {
        let mut set = MetricSet::new();
        let a = set.resolve(&SmallStr::from_static("a"));
        let b = set.resolve(&SmallStr::from_static("b"));
        let c = set.resolve(&SmallStr::from_static("c"));
        assert_eq!(a.get(), 0);
        assert_eq!(b.get(), 1);
        assert_eq!(c.get(), 2);
    }
}

// fn add_or_update_internal(&mut self, generation: u64, mut metric: Metric) {
//     self.meta.update_count += 1;
//     if let Some(&idx) = self.by_name.get(metric.name().as_str()) {
//         let existing = &mut self.metrics[idx.as_usize()];
//         existing.set_generation(generation);
//         existing.update_from(metric);
//     } else {
//         metric.set_generation(generation);
//         let idx = MetricIdx::new(self.metrics.len() as u32);
//         self.by_name.insert(metric.name().clone(), idx);
//         self.metrics.push(metric);
//     }
// }
