use crate::{
    Metric, MetricUpdate,
    stats::{Tag, TagKind, defaults::try_add_tag_from_str, fmt},
};
use radiate_utils::intern;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

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
            set_stats: Metric::new(METRIC_SET),
        }
    }

    #[inline(always)]
    pub fn keys(&self) -> Vec<&'static str> {
        self.metrics.keys().cloned().collect()
    }

    #[inline(always)]
    pub fn flush_all_into(&mut self, target: &mut MetricSet) {
        for (key, mut m) in self.metrics.drain() {
            if let Some(target_metric) = target.metrics.get_mut(key) {
                target_metric.update_from(m);
            } else {
                try_add_tag_from_str(&mut m);
                target.metrics.insert(key, m);
            }
        }

        target.set_stats.update_from(self.set_stats.clone());
        self.clear();
    }

    #[inline(always)]
    pub fn upsert<'a>(&mut self, metric: impl Into<MetricSetUpdate<'a>>) {
        let update = metric.into();
        match update {
            MetricSetUpdate::Many(metrics) => {
                for metric in metrics {
                    self.add_or_update_internal(metric);
                }
            }
            MetricSetUpdate::Single(metric) => {
                self.add_or_update_internal(metric);
            }
            MetricSetUpdate::NamedSingle(name, metric_update) => {
                self.set_stats.apply_update(1);
                if let Some(m) = self.metrics.get_mut(name) {
                    m.apply_update(metric_update);
                    return;
                }

                let new_name = radiate_utils::intern_name_as_snake_case(name);
                if let Some(m) = self.metrics.get_mut(&new_name) {
                    m.apply_update(metric_update);
                } else {
                    let mut metric = Metric::new(new_name);
                    metric.apply_update(metric_update);
                    self.add(metric);
                }
            }
        }
    }

    fn add_or_update_internal(&mut self, metric: Metric) {
        self.set_stats.apply_update(1);
        if let Some(existing) = self.metrics.get_mut(metric.name()) {
            existing.update_from(metric);
        } else {
            self.metrics.insert(intern!(metric.name()), metric);
        }
    }

    pub fn iter_tagged<'a>(
        &'a self,
        tag: TagKind,
    ) -> impl Iterator<Item = (&'static str, &'a Metric)> {
        self.metrics
            .iter()
            .filter_map(move |(k, m)| if m.tags.has(tag) { Some((*k, m)) } else { None })
    }

    pub fn iter_stats<'a>(&'a self) -> impl Iterator<Item = &'a Metric> {
        self.metrics.values().filter(|m| m.statistic().is_some())
    }

    pub fn iter_times<'a>(&'a self) -> impl Iterator<Item = &'a Metric> {
        self.metrics
            .values()
            .filter(|m| m.time_statistic().is_some())
    }

    pub fn tags(&self) -> impl Iterator<Item = TagKind> {
        self.metrics
            .values()
            .fold(Tag::empty(), |acc, m| acc.union(m.tags))
            .into_iter()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, &Metric)> {
        self.metrics.iter().map(|(name, metric)| (*name, metric))
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
        fmt::render_full(self).unwrap_or_default()
    }

    // --- Default accessors ---
    pub fn time(&self) -> Option<&Metric> {
        self.get(super::metric_names::TIME)
    }

    pub fn score(&self) -> Option<&Metric> {
        self.get(super::metric_names::SCORES)
    }

    pub fn improvements(&self) -> Option<&Metric> {
        self.get(super::metric_names::BEST_SCORE_IMPROVEMENT)
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

    pub fn front_size(&self) -> Option<&Metric> {
        self.get(super::metric_names::FRONT_SIZE)
    }

    pub fn front_comparisons(&self) -> Option<&Metric> {
        self.get(super::metric_names::FRONT_COMPARISONS)
    }

    pub fn front_removals(&self) -> Option<&Metric> {
        self.get(super::metric_names::FRONT_REMOVALS)
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
        let metrics = Vec::<Metric>::deserialize(deserializer)?;

        let mut metric_set = MetricSet::new();
        for metric in metrics {
            metric_set.add(metric);
        }

        Ok(metric_set)
    }
}

pub enum MetricSetUpdate<'a> {
    Many(Vec<Metric>),
    Single(Metric),
    NamedSingle(&'static str, MetricUpdate<'a>),
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
        MetricSetUpdate::NamedSingle(name, update.into())
    }
}
