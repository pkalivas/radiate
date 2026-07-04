use crate::{
    Metric, MetricUpdate,
    stats::{Meta, Tag, TagType, fmt},
};
pub use radiate_expr::*;
use radiate_utils::{AnyValue, SmallStr};
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
pub(crate) struct MetricIdx(u32);

impl MetricIdx {
    #[inline(always)]
    pub(crate) const fn new(idx: u32) -> Self {
        MetricIdx(idx)
    }

    #[inline(always)]
    pub(crate) const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MetricQueryIndex {
    pub metric: MetricIdx,
    pub name: SmallStr,
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

    pub fn bump(&mut self, generation: u64) {
        self.meta.generation = generation;
    }

    pub fn generation(&self) -> u64 {
        self.meta.generation
    }

    pub(crate) fn register(&mut self, name: impl Into<SmallStr>) -> MetricQueryIndex {
        let name = name.into();
        let idx = self.resolve(&name);
        MetricQueryIndex { metric: idx, name }
    }

    /// Resolve a name to a stable [`MetricIdx`], registering an empty metric if
    /// the name has not been seen before. The returned handle is valid for the
    /// lifetime of this `MetricSet`.
    #[inline]
    pub(crate) fn resolve(&mut self, name: impl AsRef<str>) -> MetricIdx {
        if let Some(&idx) = self.name_lookup.get(name.as_ref()) {
            return idx;
        }

        let idx = MetricIdx::new(self.metrics.len() as u32);
        let name = SmallStr::from(name.as_ref());
        self.name_lookup.insert(name.clone(), idx);
        self.metrics.push(Metric::new(name));
        idx
    }

    #[inline]
    pub(crate) fn upsert_at<'a>(&mut self, idx: MetricIdx, update: impl Into<MetricUpdate<'a>>) {
        let generation = self.meta.generation;
        let mmetric = &mut self.metrics[idx.as_usize()];

        mmetric.set_generation(generation);
        mmetric.apply_update(update.into());

        self.meta.update_count += 1;
    }

    #[inline(always)]
    pub fn upsert<'a>(&mut self, key: impl AsRef<str>, metric: impl Into<MetricUpdate<'a>>) {
        let metric_update = metric.into();
        let idx = self.resolve(&key);
        self.upsert_at(idx, metric_update);
    }

    #[inline(always)]
    pub fn upsert_tagged<'a>(
        &mut self,
        key: impl AsRef<str>,
        metric: impl Into<MetricUpdate<'a>>,
        tag: TagType,
    ) {
        let metric_update = metric.into();
        let idx = self.resolve(&key);
        if let Some(metric) = self.metrics.get_mut(idx.as_usize()) {
            metric.add_tag(tag);
            self.upsert_at(idx, metric_update);
        }
    }

    #[inline(always)]
    pub fn keys(&self) -> impl Iterator<Item = SmallStr> {
        self.metrics.iter().map(|m| m.name().clone())
    }

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
    pub fn iter_tagged(&self, tag: TagType) -> impl Iterator<Item = &Metric> {
        self.metrics.iter().filter(move |m| m.tags().has(tag))
    }

    #[inline(always)]
    pub fn tags(&self) -> impl Iterator<Item = TagType> {
        self.metrics
            .iter()
            .fold(Tag::empty(), |acc, m| acc.union(m.tags()))
            .into_iter()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &Metric> {
        self.metrics.iter()
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

    pub fn remove_samples(&mut self) {
        for m in &mut self.metrics {
            if m.tags().has(TagType::Distribution) {
                m.clear_samples();
            }
        }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.metrics.is_empty()
    }

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

impl ExprSelector for MetricSet {
    fn select(&self, sel: &SelectExpr) -> AnyValue<'static> {
        // Missing metrics return Null so downstream math can propagate it; the
        // outer Clamp (or any consumer using non-finite fallback) then takes the
        // floor instead of the engine seeing an unrelated error.
        let Some(metric) = sel.metric.as_ref().and_then(|name| self.get(name.as_str())) else {
            return AnyValue::Null;
        };

        let wrap = |v: f32| match sel.kind {
            MetricKind::Value => AnyValue::Float32(v),
            MetricKind::Duration => AnyValue::Duration(Duration::from_secs_f32(v)),
        };

        match sel.field {
            MetricField::LastValue => wrap(metric.last_value()),
            MetricField::Mean => wrap(metric.mean()),
            MetricField::StdDev => wrap(metric.stddev()),
            MetricField::Min => wrap(metric.min()),
            MetricField::Max => wrap(metric.max()),
            MetricField::Sum => wrap(metric.sum()),
            MetricField::Var => wrap(metric.var()),
            MetricField::Skew => AnyValue::Float32(metric.skew()),
            MetricField::Count => AnyValue::UInt64(metric.count() as u64),
            MetricField::Generation => AnyValue::UInt64(metric.generation()),
            MetricField::UpdateCount => AnyValue::UInt64(metric.update_count() as u64),
        }
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
    Single(SmallStr, MetricUpdate<'a>, Option<TagType>),
}

impl<'a, N, U> From<(N, U)> for MetricSetUpdate<'a>
where
    N: Into<SmallStr>,
    U: Into<MetricUpdate<'a>>,
{
    fn from((name, update): (N, U)) -> Self {
        MetricSetUpdate::Single(name.into(), update.into(), None)
    }
}

impl<'a, N, U> From<(TagType, N, U)> for MetricSetUpdate<'a>
where
    N: Into<SmallStr>,
    U: Into<MetricUpdate<'a>>,
{
    fn from((tag, name, update): (TagType, N, U)) -> Self {
        MetricSetUpdate::Single(name.into(), update.into(), Some(tag))
    }
}

impl<'a, N, U> From<(N, U, usize)> for MetricSetUpdate<'a>
where
    N: AsRef<str>,
    U: Into<MetricUpdate<'a>>,
{
    fn from((name, update, count): (N, U, usize)) -> Self {
        let name: SmallStr = format!("{}.{}", name.as_ref(), count).into();
        MetricSetUpdate::Single(name, update.into(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn resolve_assigns_sequential_indices() {
        let mut set = MetricSet::new();
        let a = set.resolve(&SmallStr::from_static("a"));
        let b = set.resolve(&SmallStr::from_static("b"));
        let c = set.resolve(&SmallStr::from_static("c"));
        assert_eq!(a.as_usize(), 0);
        assert_eq!(b.as_usize(), 1);
        assert_eq!(c.as_usize(), 2);
    }
}
