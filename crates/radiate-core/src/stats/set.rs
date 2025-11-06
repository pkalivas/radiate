use crate::{Metric, MetricScope, MetricUpdate, Rollup, intern, stats::fmt};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

pub(super) const METRIC_SET: &str = "metric_set";

pub struct MetricSetSummary {
    pub metrics: usize,
    pub updates: f32,
}

#[derive(Clone)]
pub struct MetricSet {
    metrics: HashMap<&'static str, Metric>,
    set_stats: Metric,
}

impl MetricSet {
    pub fn new() -> Self {
        MetricSet {
            metrics: HashMap::new(),
            set_stats: Metric::new_scoped(METRIC_SET, MetricScope::Lifetime)
                .with_rollup(Rollup::Sum),
        }
    }

    #[inline(always)]
    pub fn keys(&self) -> Vec<&'static str> {
        self.metrics.keys().cloned().collect()
    }

    #[inline(always)]
    pub fn flush_all_into(&self, target: &mut MetricSet) {
        for (_, m) in self.iter() {
            self.flush_metric_into(m.name(), target);
        }

        target.set_stats.update_from(&self.set_stats);
    }

    #[inline(always)]
    pub fn flush_scope_into(&self, from_scope: MetricScope, target: &mut MetricSet) {
        for (_, m) in self.iter_scope(from_scope) {
            self.flush_metric_into(m.name(), target);
        }
    }

    #[inline(always)]
    pub fn flush_metric_into(&self, name: &'static str, target: &mut MetricSet) {
        if let Some(m) = self.metrics.get(name) {
            let dest = target.metrics.entry(name).or_insert_with(|| {
                let mut clone = m.clone();
                clone.clear_values();
                clone
            });

            dest.update_from(m);
        }
    }

    pub fn upsert<'a>(&mut self, name: &'static str, update: impl Into<MetricUpdate<'a>>) {
        if let Some(m) = self.metrics.get_mut(name) {
            self.set_stats.apply_update(1);
            m.apply_update(update);
            return;
        }

        let new_name = super::normalize_name(name);
        if let Some(m) = self.metrics.get_mut(&new_name) {
            self.set_stats.apply_update(1);
            m.apply_update(update);
        } else {
            self.add(
                Metric::new_scoped(new_name, super::defaults::default_scope(new_name))
                    .with_rollup(super::defaults::default_rollup(new_name)),
            );

            self.set_stats.apply_update(1);
            self.metrics
                .get_mut(&new_name)
                .unwrap()
                .apply_update(update);
        }
    }

    #[inline(always)]
    pub fn add_or_update<'a>(&mut self, metric: Metric) {
        self.set_stats.apply_update(1);
        if let Some(m) = self.metrics.get_mut(metric.name()) {
            m.update_from(&metric);
        } else {
            self.add(metric);
        }
    }

    #[inline(always)]
    pub fn iter_scope(&self, scope: MetricScope) -> impl Iterator<Item = (&'static str, &Metric)> {
        self.metrics
            .iter()
            .filter_map(move |(k, m)| (m.scope() == scope).then_some((*k, m)))
    }

    #[inline(always)]
    pub fn iter_scope_mut(
        &mut self,
        scope: MetricScope,
    ) -> impl Iterator<Item = (&'static str, &mut Metric)> {
        self.metrics
            .iter_mut()
            .filter_map(move |(k, m)| (m.scope() == scope).then_some((*k, m)))
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &Metric)> {
        self.metrics.iter().map(|(name, metric)| (*name, metric))
    }

    #[inline(always)]
    pub fn clear_scope(&mut self, scope: MetricScope) {
        for (_, m) in self.iter_scope_mut(scope) {
            m.clear_values();
        }
    }

    #[inline(always)]
    pub fn add(&mut self, metric: Metric) {
        self.metrics.insert(metric.name(), metric);
    }

    #[inline(always)]
    pub fn get(&self, name: &'static str) -> Option<&Metric> {
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

        self.set_stats.clear_values();
    }

    #[inline(always)]
    pub fn contains_key(&self, name: impl Into<String>) -> bool {
        self.metrics.contains_key(intern!(name.into()))
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.metrics.len()
    }

    #[inline(always)]
    pub fn summary(&self) -> MetricSetSummary {
        MetricSetSummary {
            metrics: self.metrics.len(),
            updates: self.set_stats.statistic().map(|s| s.sum()).unwrap_or(0.0),
        }
    }
}

impl Default for MetricSet {
    fn default() -> Self {
        MetricSet::new()
    }
}

impl std::fmt::Display for MetricSet {
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
        write!(f, "MetricSet {{\n")?;
        write!(f, "{}\n", fmt::render_dashboard(&self).unwrap_or_default())?;
        write!(f, "}}")
    }
}

#[cfg(feature = "serde")]
impl Serialize for MetricSet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let metrics = self
            .metrics
            .iter()
            .map(|(_, metric)| metric.clone())
            .collect::<Vec<Metric>>();
        metrics.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for MetricSet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use crate::stats::MetricInner;

        #[derive(Deserialize)]
        struct MetricOwned {
            name: String,
            inner: MetricInner,
            scope: MetricScope,
            rollup: Rollup,
        }

        let metrics = Vec::<MetricOwned>::deserialize(deserializer)?;

        let mut metric_set = MetricSet::new();
        for metric in metrics {
            let metric = Metric {
                name: intern!(metric.name),
                inner: metric.inner,
                scope: metric.scope,
                rollup: metric.rollup,
            };
            metric_set.add(metric);
        }
        Ok(metric_set)
    }
}
