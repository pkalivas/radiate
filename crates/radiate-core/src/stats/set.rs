use crate::{
    Distribution, Metric, MetricUpdate, Rollup, Statistic, TimeStatistic,
    stats::{Tag, fmt},
};
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
            set_stats: Metric::new(METRIC_SET),
        }
    }

    #[inline(always)]
    pub fn keys(&self) -> Vec<&'static str> {
        self.metrics.keys().cloned().collect()
    }

    #[inline(always)]
    pub fn flush_all_into(&mut self, target: &mut MetricSet) {
        for (key, m) in self.metrics.drain() {
            let dest = target.metrics.entry(key).or_insert_with(|| {
                let mut clone = m.clone();
                clone.clear_values();
                clone
            });

            dest.update_from(&m);
        }

        target.set_stats.update_from(&self.set_stats);
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
            MetricSetUpdate::Slice6(metrics) => {
                for metric in metrics {
                    self.add_or_update_internal(metric);
                }
            }
            MetricSetUpdate::Slice7(metrics) => {
                for metric in metrics {
                    self.add_or_update_internal(metric);
                }
            }
            MetricSetUpdate::Fn(func) => {
                func(self);
            }
            MetricSetUpdate::NamedSingle(name, metric_update) => {
                if let Some(m) = self.metrics.get_mut(name) {
                    self.set_stats.apply_update(1);
                    m.apply_update(metric_update);
                    return;
                }

                let new_name = radiate_utils::intern_name_as_snake_case(name);
                if let Some(m) = self.metrics.get_mut(&new_name) {
                    self.set_stats.apply_update(1);
                    m.apply_update(metric_update);
                } else {
                    self.add(
                        Metric::new(new_name)
                            .with_rollup(super::defaults::default_rollup(new_name)),
                    );

                    self.set_stats.apply_update(1);
                    self.metrics
                        .get_mut(&new_name)
                        .unwrap()
                        .apply_update(metric_update);
                }
            }
        }
    }

    #[inline(always)]
    pub fn iter_tagged<'a>(
        &'a self,
        tag: impl Into<Tag>,
    ) -> impl Iterator<Item = (&'static str, &'a Metric)> {
        let tag = tag.into();
        self.metrics.iter().filter_map(move |(k, m)| {
            if let Some(tags) = m.tags() {
                if tags.iter().any(|t| t.0 == tag.0) {
                    return Some((*k, m));
                }
            }

            None
        })
    }

    #[inline(always)]
    pub fn iter_stats<'a>(&'a self) -> impl Iterator<Item = &'a Metric> {
        self.metrics.values().filter(|m| m.statistic().is_some())
    }

    #[inline(always)]
    pub fn iter_distributions<'a>(&'a self) -> impl Iterator<Item = &'a Metric> {
        self.metrics.values().filter(|m| m.distribution().is_some())
    }

    #[inline(always)]
    pub fn iter_times<'a>(&'a self) -> impl Iterator<Item = &'a Metric> {
        self.metrics
            .values()
            .filter(|m| m.time_statistic().is_some())
    }

    #[inline(always)]
    pub fn tags(&self) -> impl Iterator<Item = &Tag> {
        let mut seen = HashMap::new();

        for (_, metric) in self.metrics.iter() {
            if let Some(tags) = metric.tags() {
                for tag in tags.iter() {
                    seen.entry(tag).or_insert(());
                }
            }
        }

        seen.into_keys()
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
            // stats: self.set_stats.
        }
    }

    pub fn sorted_tagged<'a>(&'a self, tag: impl Into<Tag>) -> Vec<(&'static str, &'a Metric)> {
        let mut items = self.iter_tagged(tag).collect::<Vec<_>>();
        items.sort_by(|a, b| a.0.cmp(b.0));
        items
    }

    pub fn get_statistic_val<F, T>(&self, name: &str, func: F) -> T
    where
        F: Fn(&Statistic) -> T,
        T: Default,
    {
        self.get(name).map(|m| m.get_stat(func)).unwrap_or_default()
    }

    pub fn get_dist_val<F, T>(&self, name: &str, func: F) -> T
    where
        F: Fn(&Distribution) -> T,
        T: Default,
    {
        self.get(name).map(|m| m.get_dist(func)).unwrap_or_default()
    }

    pub fn get_time_val<F, T>(&self, name: &str, func: F) -> T
    where
        F: Fn(&TimeStatistic) -> T,
        T: Default,
    {
        self.get(name).map(|m| m.get_time(func)).unwrap_or_default()
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
        use std::sync::Arc;

        use crate::stats::MetricInner;

        #[derive(Deserialize)]
        struct MetricOwned {
            name: String,
            inner: MetricInner,
            rollup: Rollup,
            tags: Option<Arc<Vec<Arc<String>>>>,
        }

        let metrics = Vec::<MetricOwned>::deserialize(deserializer)?;

        let mut metric_set = MetricSet::new();
        for metric in metrics {
            let metric = Metric {
                name: metric.name.into(),
                inner: metric.inner,
                rollup: metric.rollup,
                tags: metric.tags.map(|tags| {
                    use crate::stats::Tag;

                    Arc::new(tags.iter().map(|tag| Tag(intern!(tag.as_str()))).collect())
                }),
            };
            metric_set.add(metric);
        }
        Ok(metric_set)
    }
}

pub enum MetricSetUpdate<'a> {
    Fn(Box<dyn FnOnce(&mut MetricSet) + Send>),
    Many(Vec<Metric>),
    NamedSingle(&'static str, MetricUpdate<'a>),
    Single(Metric),
    Slice2([Metric; 2]),
    Slice3([Metric; 3]),
    Slice4([Metric; 4]),
    Slice5([Metric; 5]),
    Slice6([Metric; 6]),
    Slice7([Metric; 7]),
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

impl From<[Metric; 2]> for MetricSetUpdate<'_> {
    fn from(metrics: [Metric; 2]) -> Self {
        MetricSetUpdate::Slice2(metrics)
    }
}

impl From<[Metric; 3]> for MetricSetUpdate<'_> {
    fn from(metrics: [Metric; 3]) -> Self {
        MetricSetUpdate::Slice3(metrics)
    }
}

impl From<[Metric; 4]> for MetricSetUpdate<'_> {
    fn from(metrics: [Metric; 4]) -> Self {
        MetricSetUpdate::Slice4(metrics)
    }
}

impl From<[Metric; 5]> for MetricSetUpdate<'_> {
    fn from(metrics: [Metric; 5]) -> Self {
        MetricSetUpdate::Slice5(metrics)
    }
}

impl<F> From<F> for MetricSetUpdate<'_>
where
    F: FnOnce(&mut MetricSet) + Send + 'static,
{
    fn from(func: F) -> Self {
        MetricSetUpdate::Fn(Box::new(func))
    }
}

impl From<[Metric; 6]> for MetricSetUpdate<'_> {
    fn from(metrics: [Metric; 6]) -> Self {
        MetricSetUpdate::Slice6(metrics)
    }
}

impl From<[Metric; 7]> for MetricSetUpdate<'_> {
    fn from(metrics: [Metric; 7]) -> Self {
        MetricSetUpdate::Slice7(metrics)
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
