use crate::stats::{MetricView, Tag, TagType, defaults};
use radiate_error::{RadiateError, radiate_err};
use radiate_expr::{AnyValue, DataType};
use radiate_utils::{
    Statistic, ToSnakeCase, cache_arc_string, intern, intern_snake_case,
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{hash::Hash, sync::Arc, time::Duration};

const DATA_TYPE_NULL: u8 = 0;
const DATA_TYPE_FLOAT32: u8 = 1;
const DATA_TYPE_DURATION: u8 = 2;
const DATA_TYPE_LIST: u8 = 3;

#[macro_export]
macro_rules! metric {
    ($name:expr, $update:expr) => {{
        let mut metric = $crate::Metric::new($name);
        metric.apply_update($update);
        metric
    }};
    ($name:expr) => {{ $crate::Metric::new($name).upsert(1) }};
}


#[derive(Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(super) struct Meta {
    pub(super) update_count: usize,
    pub(super) version: u64,
}

#[derive(Clone, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Metric {
    name: Arc<String>,
    meta: Option<Meta>,
    inner: Statistic,
    tags: Tag,
    dtype: u8,
}

impl Metric {
    pub fn new(name: &'static str) -> Self {
        let name = cache_arc_string!(intern_snake_case!(name));
        let tags = defaults::default_tags(&name);

        Self {
            name,
            meta: None,
            inner: Statistic::default(),
            tags,
            dtype: DATA_TYPE_NULL,
        }
    }

    #[inline(always)]
    pub fn update_count(&self) -> usize {
        self.meta.as_ref().map_or(0, |meta| meta.update_count)
    }

    #[inline(always)]
    pub fn version(&self) -> u64 {
        self.meta.as_ref().map_or(0, |meta| meta.version)
    }

    #[inline(always)]
    pub fn set_version(&mut self, version: u64) {
        if let Some(meta) = &mut self.meta {
            if version != meta.version {
                meta.update_count = 0;
            }

            meta.version = version;
        } else {
            self.meta = Some(Meta {
                update_count: 0,
                version,
            });
        }
    }

    pub fn dtype(&self) -> DataType {
        match self.dtype {
            DATA_TYPE_NULL => DataType::Null,
            DATA_TYPE_FLOAT32 => DataType::Float32,
            DATA_TYPE_DURATION => DataType::Duration,
            DATA_TYPE_LIST => DataType::List(Box::new(DataType::Float32)),
            _ => DataType::Null,
        }
    }

    #[inline(always)]
    pub fn tags(&self) -> Tag {
        self.tags
    }

    #[inline(always)]
    pub fn with_tag(mut self, tag: TagType) -> Self {
        self.add_tag(tag);
        self
    }

    #[inline(always)]
    pub fn with_tags<T>(&mut self, tags: T)
    where
        T: Into<Tag>,
    {
        self.tags = tags.into();
    }

    #[inline(always)]
    pub fn add_tags(&mut self, tags: Tag) {
        self.tags = self.tags.union(tags);
    }

    #[inline(always)]
    pub fn add_tag(&mut self, tag: TagType) {
        self.tags.insert(tag);
    }

    pub fn contains_tag(&self, tag: &TagType) -> bool {
        self.tags.has(*tag)
    }

    pub fn tags_iter(&self) -> impl Iterator<Item = TagType> {
        self.tags.iter()
    }

    pub fn clear_values(&mut self) {
        self.inner = Statistic::default();
    }

    pub fn stats<'a>(&'a self) -> Option<MetricView<'a, f32>> {
        if !self.tags.has(TagType::Statistic) {
            return None;
        }

        Some(MetricView {
            name: &self.name,
            statistic: &self.inner,
            mapper: |v| v,
        })
    }

    pub fn times<'a>(&'a self) -> Option<MetricView<'a, Duration>> {
        if !self.tags.has(TagType::Time) {
            return None;
        }

        Some(MetricView {
            name: &self.name,
            statistic: &self.inner,
            mapper: |v| Duration::from_secs_f32(v),
        } )
    }

    pub fn distributions<'a>(&'a self) -> Option<MetricView<'a, f32>> {
        if !self.tags.has(TagType::Distribution) {
            return None;
        }

        Some(MetricView {
            name: &self.name,
            statistic: &self.inner,
            mapper: |v| v,
         })
    }

    #[inline(always)]
    pub fn upsert<'a>(mut self, update: impl Into<MetricUpdate<'a>>) -> Self {
        self.apply_update(update);
        self
    }

    #[inline(always)]
    pub fn update_from(&mut self, other: Metric) {
        // Kinda a hack to take advantage of the fact that if count == sum,
        // we can just apply the sum directly instead of merging statistics - keeps things honest
        // & avoids merging statistics when we don't have to (even though that's a fast operation).
        if other.count() as f32 == other.sum() && !other.tags.has(TagType::Distribution) {
            self.apply_update(other.sum());
        } else {
            self.apply_update(other.inner);
        }

        self.tags = self.tags.union(other.tags);
    }

    #[inline(always)]
    pub fn apply_update<'a>(&mut self, update: impl Into<MetricUpdate<'a>>) {
        let update = update.into();
        match update {
            MetricUpdate::Float(value) => {
                self.update_statistic(value);
            }
            MetricUpdate::Usize(value) => {
                self.update_statistic(value as f32);
            }
            MetricUpdate::Duration(value) => {
                self.update_time_statistic(value);
            }
            MetricUpdate::UsizeDistribution(values) => {
                self.update_statistic_from_iter(values.iter().map(|v| *v as f32));
            }
            MetricUpdate::Distribution(values) => {
                self.update_statistic_from_iter(values.iter().cloned());
            }
            MetricUpdate::Statistic(stat) => {
                self.inner.merge(&stat);                
                self.meta.as_mut().map(|meta| meta.update_count += 1);
                self.dtype = DATA_TYPE_FLOAT32;
            }
        }
    }

    fn update_statistic(&mut self, value: f32) {
        self.inner.add(value);
        self.add_tag(TagType::Statistic);
        self.meta.as_mut().map(|meta| meta.update_count += 1);

        if self.dtype == DATA_TYPE_NULL {
            self.dtype = DATA_TYPE_FLOAT32;
        }
    }

    fn update_time_statistic(&mut self, value: Duration) {
        self.inner.add(value.as_secs_f32());
        self.add_tag(TagType::Time);
        self.meta.as_mut().map(|meta| meta.update_count += 1);

        if self.dtype == DATA_TYPE_NULL {
            self.dtype = DATA_TYPE_DURATION;
        }
    }

    fn update_statistic_from_iter<I>(&mut self, values: I)
    where
        I: IntoIterator<Item = f32>,
    {   
        let mut values_count = 0;
        let mut new_stat = Statistic::default();
        for value in values {
            new_stat.add(value);
            values_count += 1;
        }
        
        self.inner = new_stat;
        self.meta.as_mut().map(|meta| meta.update_count += values_count);
        self.add_tag(TagType::Distribution);

        if self.dtype == DATA_TYPE_NULL {
            self.dtype = DATA_TYPE_LIST;
        }
    }

    pub fn statistic(&self) -> &Statistic {
        &self.inner
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn last_value(&self) -> f32 {
        self.inner.last_value()
    }

    pub fn count(&self) -> i32 {
        self.inner.count()
    }

    pub fn mean(&self) -> f32 {
        self.inner.mean()
    }

    pub fn var(&self) -> f32 {
        self.inner.variance().unwrap_or(0.0)
    }

    pub fn stddev(&self) -> f32 {
        self.inner.std_dev().unwrap_or(0.0)
    }

    pub fn skew(&self) -> f32 {
        self.inner.skewness().unwrap_or(0.0)
    }

    pub fn min(&self) -> f32 {
        self.inner.min()
    }

    pub fn max(&self) -> f32 {
        self.inner.max()
    }

    pub fn sum(&self) -> f32 {
        self.inner.sum()
    }
}

impl Hash for Metric {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.inner.hash(state);
        self.tags.hash(state);
    }
}

#[derive(Clone, PartialEq)]
pub enum MetricUpdate<'a> {
    Float(f32),
    Usize(usize),
    Duration(Duration),
    Distribution(&'a [f32]),
    UsizeDistribution(&'a [usize]),
    Statistic(Statistic),
}

impl From<f32> for MetricUpdate<'_> {
    fn from(value: f32) -> Self {
        MetricUpdate::Float(value)
    }
}

impl From<usize> for MetricUpdate<'_> {
    fn from(value: usize) -> Self {
        MetricUpdate::Usize(value)
    }
}

impl From<Duration> for MetricUpdate<'_> {
    fn from(value: Duration) -> Self {
        MetricUpdate::Duration(value)
    }
}

impl<'a> From<&'a [f32]> for MetricUpdate<'a> {
    fn from(value: &'a [f32]) -> Self {
        MetricUpdate::Distribution(value)
    }
}

impl<'a> From<&'a Vec<f32>> for MetricUpdate<'a> {
    fn from(value: &'a Vec<f32>) -> Self {
        MetricUpdate::Distribution(value)
    }
}

impl<'a> From<&'a Vec<usize>> for MetricUpdate<'a> {
    fn from(value: &'a Vec<usize>) -> Self {
        MetricUpdate::UsizeDistribution(value)
    }
}

impl From<Statistic> for MetricUpdate<'_> {
    fn from(value: Statistic) -> Self {
        MetricUpdate::Statistic(value)
    }
}

impl<'a> TryFrom<AnyValue<'a>> for MetricUpdate<'a> {
    type Error = RadiateError;

    fn try_from(value: AnyValue<'a>) -> Result<Self, Self::Error> {
        match value {
            AnyValue::UInt8(v) => Ok(MetricUpdate::Float(v as f32)),
            AnyValue::UInt16(v) => Ok(MetricUpdate::Float(v as f32)),
            AnyValue::UInt32(v) => Ok(MetricUpdate::Float(v as f32)),
            AnyValue::UInt64(v) => Ok(MetricUpdate::Float(v as f32)),
            AnyValue::UInt128(v) => Ok(MetricUpdate::Float(v as f32)),

            AnyValue::Int8(v) => Ok(MetricUpdate::Float(v as f32)),
            AnyValue::Int16(v) => Ok(MetricUpdate::Float(v as f32)),
            AnyValue::Int32(v) => Ok(MetricUpdate::Float(v as f32)),
            AnyValue::Int64(v) => Ok(MetricUpdate::Float(v as f32)),
            AnyValue::Int128(v) => Ok(MetricUpdate::Float(v as f32)),

            AnyValue::Float32(v) => Ok(MetricUpdate::Float(v)),
            AnyValue::Float64(v) => Ok(MetricUpdate::Float(v as f32)),

            AnyValue::Duration(v) => Ok(MetricUpdate::Duration(v)),

            AnyValue::Slice(values) => {
                let out = values
                    .iter()
                    .enumerate()
                    .map(|(index, v)| {
                        v.clone().extract::<f32>().ok_or(
                            radiate_err!(
                                Metric: 
                                "cannot convert AnyValue sequence into MetricUpdate::Statistic: element at index {index} has non-numeric type `{}`", v.type_name()))
                            
                    })
                    .collect::<Result<Statistic, _>>()?;

                Ok(MetricUpdate::Statistic(out))
            }

            AnyValue::Vector(values) => {
                let out = values
                    .into_iter()
                    .enumerate()
                    .map(|(index, v)| {
                        let ty = v.type_name();
                        v.extract::<f32>()
                            .ok_or(radiate_err!(
                                Metric: 
                                "cannot convert AnyValue sequence into MetricUpdate::Distribution: element at index {index} has non-numeric type `{ty}`"
                            ))
                    })
                    .collect::<Result<Statistic, _>>()?;

                Ok(MetricUpdate::Statistic(out))
            }

            other => Err(radiate_err!(Metric: "cannot convert AnyValue of type `{}` into MetricUpdate", other.type_name())),
        }
    }
}

impl std::fmt::Debug for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Metric {{ name: {}, }}", self.name)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }

    fn assert_stat_eq(m: &Metric, count: i32, mean: f32, var: f32, min: f32, max: f32) {
        assert_eq!(m.count(), count);
        assert!(approx_eq(m.mean(), mean, EPSILON), "mean");
        assert!(approx_eq(m.var(), var, EPSILON), "var");
        assert!(approx_eq(m.min(), min, EPSILON), "min");
        assert!(approx_eq(m.max(), max, EPSILON), "max");
    }

    fn stats_of(values: &[f32]) -> (i32, f32, f32, f32, f32) {
        // sample variance (n-1), matches your Statistic::variance
        let n = values.len() as i32;
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
    fn test_metric() {
        let mut metric = Metric::new("test");
        metric.apply_update(1.0);
        metric.apply_update(2.0);
        metric.apply_update(3.0);
        metric.apply_update(4.0);
        metric.apply_update(5.0);

        assert_eq!(metric.count(), 5);
        assert_eq!(metric.last_value(), 5.0);
        assert_eq!(metric.mean(), 3.0);
        assert_eq!(metric.var(), 2.5);
        assert_eq!(metric.stddev(), 1.5811388);
        assert_eq!(metric.min(), 1.0);
        assert_eq!(metric.max(), 5.0);
        assert_eq!(metric.name(), "test");
    }

    #[test]
    fn test_metric_labels() {
        let mut metric = Metric::new("test");

        metric.apply_update(1.0);
        metric.apply_update(2.0);
        metric.apply_update(3.0);
        metric.apply_update(4.0);
        metric.apply_update(5.0);

        assert_eq!(metric.count(), 5);
        assert_eq!(metric.last_value(), 5.0);
        assert_eq!(metric.mean(), 3.0);
        assert_eq!(metric.var(), 2.5);
        assert_eq!(metric.stddev(), 1.5811388);
        assert_eq!(metric.min(), 1.0);
        assert_eq!(metric.max(), 5.0);
    }

    #[test]
    fn distribution_tag_is_applied_on_any_slice_update() {
        let mut m = Metric::new("scores");

        // seed with scalar samples first (creates Statistic but not Distribution tag)
        m.apply_update(1.0);
        m.apply_update(2.0);
        assert!(m.tags().has(TagType::Statistic));
        assert!(!m.tags().has(TagType::Distribution));

        // now apply a slice update - we expect Distribution tag to appear
        m.apply_update(&[3.0, 4.0][..]);

        assert!(
            m.tags().has(TagType::Distribution),
            "expected Distribution tag after slice update"
        );
    }

    #[test]
    fn metric_merge_matches_streaming_samples() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [10.0, 20.0, 30.0];

        let mut m1 = Metric::new("x");
        m1.apply_update(&a[..]);

        let mut m2 = Metric::new("x");
        m2.apply_update(&b[..]);

        m1.update_from(m2);

        let combined = [1.0, 2.0, 3.0, 4.0, 10.0, 20.0, 30.0];
        let (n, mean, var, min, max) = stats_of(&combined);
        assert_stat_eq(&m1, n, mean, var, min, max);
    }
}
