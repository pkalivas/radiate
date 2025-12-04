use crate::{Metric, MetricScope, MetricUpdate, Rollup, stats::fmt};
use radiate_utils::intern;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug};

pub(super) const METRIC_SET: &str = "metric_set";

pub struct MetricSetSummary {
    pub metrics: usize,
    pub updates: f32,
}

#[derive(Clone, Default)]
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
    pub fn flush_all_into(&mut self, target: &mut MetricSet) {
        for (_, m) in self.iter() {
            self.flush_metric_into(m.name(), target);
        }

        target.set_stats.update_from(&self.set_stats);
        self.clear();
    }

    #[inline(always)]
    pub fn flush_metric_into(&self, name: &str, target: &mut MetricSet) {
        if let Some(m) = self.metrics.get(name) {
            let dest = target.metrics.entry(intern!(name)).or_insert_with(|| {
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

        let new_name = radiate_utils::intern_name_as_snake_case(name);
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
    pub fn add_or_update(&mut self, metric: impl Into<MetricSetUpdate>) {
        let update = metric.into();
        match update {
            MetricSetUpdate::Many(metrics) => {
                for metric in metrics {
                    self.add_or_update_internal(metric);
                }
            }
            MetricSetUpdate::Slice2(metrics) => {
                for metric in metrics {
                    self.add_or_update_internal(metric);
                }
            }
            MetricSetUpdate::Slice3(metrics) => {
                for metric in metrics {
                    self.add_or_update_internal(metric);
                }
            }
            MetricSetUpdate::Slice4(metrics) => {
                for metric in metrics {
                    self.add_or_update_internal(metric);
                }
            }
            MetricSetUpdate::Slice5(metrics) => {
                for metric in metrics {
                    self.add_or_update_internal(metric);
                }
            }
            MetricSetUpdate::Single(metric) => {
                self.add_or_update_internal(metric);
            }
            MetricSetUpdate::Fn(func) => {
                func(self);
            }
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
        self.metrics.insert(intern!(metric.name()), metric);
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

        self.set_stats.clear_values();
    }

    #[inline(always)]
    pub fn contains_key(&self, name: &str) -> bool {
        self.metrics.contains_key(intern!(name))
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

    pub fn dashboard(&self) -> String {
        fmt::render_full(self, true).unwrap_or_default()
    }

    // --- Default accessors ---
    pub fn time(&self) -> Option<&Metric> {
        self.get(super::metric_names::TIME)
    }

    pub fn score(&self) -> Option<&Metric> {
        self.get(super::metric_names::SCORES)
    }

    pub fn age(&self) -> Option<&Metric> {
        self.get(super::metric_names::AGE)
    }

    pub fn replace_age(&self) -> Option<&Metric> {
        self.get(super::metric_names::REPLACE_AGE)
    }

    pub fn replace_invalid(&self) -> Option<&Metric> {
        self.get(super::metric_names::REPLACE_INVALID)
    }

    pub fn genome_size(&self) -> Option<&Metric> {
        self.get(super::metric_names::GENOME_SIZE)
    }

    pub fn front_additions(&self) -> Option<&Metric> {
        self.get(super::metric_names::FRONT_ADDITIONS)
    }

    pub fn front_entropy(&self) -> Option<&Metric> {
        self.get(super::metric_names::FRONT_ENTROPY)
    }

    pub fn unique_members(&self) -> Option<&Metric> {
        self.get(super::metric_names::UNIQUE_MEMBERS)
    }

    pub fn unique_scores(&self) -> Option<&Metric> {
        self.get(super::metric_names::UNIQUE_SCORES)
    }

    pub fn new_children(&self) -> Option<&Metric> {
        self.get(super::metric_names::NEW_CHILDREN)
    }

    pub fn survivor_count(&self) -> Option<&Metric> {
        self.get(super::metric_names::SURVIVOR_COUNT)
    }

    pub fn carryover_rate(&self) -> Option<&Metric> {
        self.get(super::metric_names::CARRYOVER_RATE)
    }

    pub fn lifetime_unique_members(&self) -> Option<&Metric> {
        self.get(super::metric_names::LIFETIME_UNIQUE_MEMBERS)
    }

    pub fn evaluation_count(&self) -> Option<&Metric> {
        self.get(super::metric_names::EVALUATION_COUNT)
    }

    pub fn diversity_ratio(&self) -> Option<&Metric> {
        self.get(super::metric_names::DIVERSITY_RATIO)
    }

    pub fn score_volatility(&self) -> Option<&Metric> {
        self.get(super::metric_names::SCORE_VOLATILITY)
    }

    pub fn species_count(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_COUNT)
    }

    pub fn species_age_fail(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_AGE_FAIL)
    }

    pub fn species_distance_dist(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_DISTANCE_DIST)
    }

    pub fn species_created(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_CREATED)
    }

    pub fn species_died(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_DIED)
    }

    pub fn species_age(&self) -> Option<&Metric> {
        self.get(super::metric_names::SPECIES_AGE)
    }

    fn add_or_update_internal(&mut self, metric: Metric) {
        self.set_stats.apply_update(1);
        if let Some(existing) = self.metrics.get_mut(metric.name()) {
            existing.update_from(&metric);
        } else {
            self.metrics.insert(intern!(metric.name()), metric);
        }
    }
}

impl std::fmt::Display for MetricSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = self.summary();
        let out = format!(
            "[{} metrics, {:.0} updates]",
            summary.metrics, summary.updates
        );
        write!(
            f,
            "{out}\n{}",
            fmt::render_full(self, true).unwrap_or_default()
        )?;
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
                name: metric.name.into(),
                inner: metric.inner,
                scope: metric.scope,
                rollup: metric.rollup,
            };
            metric_set.add(metric);
        }
        Ok(metric_set)
    }
}

pub enum MetricSetUpdate {
    Fn(Box<dyn FnOnce(&mut MetricSet) + Send>),
    Many(Vec<Metric>),
    Single(Metric),
    Slice2([Metric; 2]),
    Slice3([Metric; 3]),
    Slice4([Metric; 4]),
    Slice5([Metric; 5]),
}

impl From<Vec<Metric>> for MetricSetUpdate {
    fn from(metrics: Vec<Metric>) -> Self {
        MetricSetUpdate::Many(metrics)
    }
}

impl From<Metric> for MetricSetUpdate {
    fn from(metric: Metric) -> Self {
        MetricSetUpdate::Single(metric)
    }
}

impl From<[Metric; 2]> for MetricSetUpdate {
    fn from(metrics: [Metric; 2]) -> Self {
        MetricSetUpdate::Slice2(metrics)
    }
}

impl From<[Metric; 3]> for MetricSetUpdate {
    fn from(metrics: [Metric; 3]) -> Self {
        MetricSetUpdate::Slice3(metrics)
    }
}

impl From<[Metric; 4]> for MetricSetUpdate {
    fn from(metrics: [Metric; 4]) -> Self {
        MetricSetUpdate::Slice4(metrics)
    }
}

impl From<[Metric; 5]> for MetricSetUpdate {
    fn from(metrics: [Metric; 5]) -> Self {
        MetricSetUpdate::Slice5(metrics)
    }
}

impl<F> From<F> for MetricSetUpdate
where
    F: FnOnce(&mut MetricSet) + Send + 'static,
{
    fn from(func: F) -> Self {
        MetricSetUpdate::Fn(Box::new(func))
    }
}
