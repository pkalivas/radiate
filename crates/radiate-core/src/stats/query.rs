use crate::stats::metrics::{Metric, MetricLabel, MetricSet};
use std::collections::HashMap;

pub enum QueryResult<'a> {
    Raw(Vec<(&'static str, &'a Metric)>),
    Grouped(HashMap<String, Vec<&'a Metric>>),
    Aggregated(HashMap<String, f32>),
}

trait IntoQueryForms<'a> {
    fn into_raw(self) -> Vec<(&'static str, &'a Metric)>;
    fn into_grouped(self) -> HashMap<String, Vec<&'a Metric>>;
    fn into_aggregated(self) -> HashMap<String, f32>;
}

impl<'a> IntoQueryForms<'a> for QueryResult<'a> {
    fn into_raw(self) -> Vec<(&'static str, &'a Metric)> {
        match self {
            QueryResult::Raw(v) => v,
            _ => panic!("Not a raw result"),
        }
    }

    fn into_grouped(self) -> HashMap<String, Vec<&'a Metric>> {
        match self {
            QueryResult::Grouped(m) => m,
            _ => panic!("Not a grouped result"),
        }
    }

    fn into_aggregated(self) -> HashMap<String, f32> {
        match self {
            QueryResult::Aggregated(m) => m,
            _ => panic!("Not an aggregated result"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    Value,
    Time,
    Distribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupBy {
    Name,
    LabelKey(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub enum Aggregate {
    Sum,
    Mean,
    Count,
}

pub struct MetricQuery<'a> {
    metric_set: &'a MetricSet,
    name_filter: Option<Box<dyn Fn(&str) -> bool + 'a>>,
    label_filter: Option<Box<dyn Fn(&MetricLabel) -> bool + 'a>>,
    type_filter: Option<MetricType>,
    group_by: Option<GroupBy>,
    aggregate: Option<Aggregate>,
}

impl<'a> MetricQuery<'a> {
    pub fn new(metric_set: &'a MetricSet) -> Self {
        Self {
            metric_set,
            name_filter: None,
            label_filter: None,
            type_filter: None,
            group_by: None,
            aggregate: None,
        }
    }

    pub fn filter_name(mut self, filter: impl Fn(&str) -> bool + 'a) -> Self {
        self.name_filter = Some(Box::new(filter));
        self
    }

    pub fn filter_label(mut self, filter: impl Fn(&MetricLabel) -> bool + 'a) -> Self {
        self.label_filter = Some(Box::new(filter));
        self
    }

    pub fn filter_type(mut self, metric_type: MetricType) -> Self {
        self.type_filter = Some(metric_type);
        self
    }

    pub fn group_by(mut self, group: GroupBy) -> Self {
        self.group_by = Some(group);
        self
    }

    pub fn aggregate(mut self, aggr: Aggregate) -> Self {
        self.aggregate = Some(aggr);
        self
    }

    pub fn as_raw(&self) -> Vec<(&'static str, &'a Metric)> {
        self.run().into_raw()
    }

    pub fn as_grouped(&self) -> HashMap<String, Vec<&'a Metric>> {
        self.run().into_grouped()
    }

    pub fn as_aggregated(&self) -> HashMap<String, f32> {
        self.run().into_aggregated()
    }

    pub fn run(&self) -> QueryResult<'a> {
        let filtered: Vec<(&'static str, &Metric)> = self
            .metric_set
            .iter()
            .filter(|(name, metric)| {
                let name_ok = self.name_filter.as_ref().map_or(true, |f| f(name));
                let type_ok = self.type_filter.map_or(true, |typ| match typ {
                    MetricType::Value => metric.inner().value_statistic.is_some(),
                    MetricType::Time => metric.inner().time_statistic.is_some(),
                    MetricType::Distribution => metric.inner().distribution.is_some(),
                });
                let label_ok = self.label_filter.as_ref().map_or(true, |f| {
                    if let Some(labels) = metric.labels() {
                        labels.iter().any(|l| f(l))
                    } else {
                        false
                    }
                });

                name_ok && type_ok && label_ok
            })
            .collect();

        match (self.group_by, self.aggregate) {
            (Some(group), Some(agg)) => {
                let mut groups: HashMap<String, Vec<&Metric>> = HashMap::new();
                for (name, metric) in filtered.iter().copied() {
                    let key = match group {
                        GroupBy::Name => name.to_string(),
                        GroupBy::LabelKey(label_key) => metric
                            .labels()
                            .and_then(|ls| ls.iter().find(|l| l.key == label_key))
                            .map(|l| l.value.clone())
                            .unwrap_or_else(|| "unknown".to_string()),
                    };
                    groups.entry(key).or_default().push(metric);
                }

                let mut result = HashMap::new();
                for (key, group) in groups {
                    let vals: Vec<f32> = group.iter().filter_map(|m| m.value_mean()).collect();
                    let value = match agg {
                        Aggregate::Sum => vals.iter().copied().sum(),
                        Aggregate::Mean => {
                            let sum: f32 = vals.iter().copied().sum();
                            if vals.len() > 0 {
                                sum / vals.len() as f32
                            } else {
                                0.0
                            }
                        }
                        Aggregate::Count => vals.len() as f32,
                    };
                    result.insert(key, value);
                }

                QueryResult::Aggregated(result)
            }
            (Some(group), None) => {
                let mut groups: HashMap<String, Vec<&Metric>> = HashMap::new();
                for (name, metric) in filtered.iter().copied() {
                    let key = match group {
                        GroupBy::Name => name.to_string(),
                        GroupBy::LabelKey(label_key) => metric
                            .labels()
                            .and_then(|ls| ls.iter().find(|l| l.key == label_key))
                            .map(|l| l.value.clone())
                            .unwrap_or_else(|| "unknown".to_string()),
                    };
                    groups.entry(key).or_default().push(metric);
                }

                QueryResult::Grouped(groups)
            }
            _ => QueryResult::Raw(filtered),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{labels, stats::metrics::*};

    #[test]
    fn test_query_by_type_and_label() {
        let mut set = MetricSet::new();
        set.upsert("time_eval", (1.0f32, std::time::Duration::from_millis(10)));
        set.upsert("value_eval", 2.5f32);
        set.upsert("selector", vec![0.2, 0.3, 0.5]);
        set.add_labels("selector", labels!["phase" => "selection"]);

        let result = MetricQuery::new(&set)
            .filter_type(MetricType::Distribution)
            .filter_label(|l| l.key == "phase")
            .run();

        let raw = result.into_raw();
        assert_eq!(raw.len(), 1);
        assert_eq!(raw[0].0, "selector");
    }

    #[test]
    fn test_aggregate_distribution_by_label() {
        let mut set = MetricSet::new();
        set.upsert("selector1", vec![0.1, 0.2, 0.7]);
        set.add_labels("selector1", labels!["phase" => "selection"]);
        set.upsert("selector2", vec![0.3, 0.4, 0.3]);
        set.add_labels("selector2", labels!["phase" => "selection"]);
        set.upsert("mutator1", vec![0.5, 0.5]);
        set.add_labels("mutator1", labels!["phase" => "mutation"]);

        let result = MetricQuery::new(&set)
            .filter_type(MetricType::Distribution)
            .group_by(GroupBy::LabelKey("phase"))
            .aggregate(Aggregate::Mean)
            .run();

        let result = result.into_aggregated();
        assert!(result.contains_key("selection"));
        assert!(result.contains_key("mutation"));
        assert_eq!(result.get("selection").unwrap().is_finite(), true);
        assert_eq!(result.get("mutation").unwrap().is_finite(), true);
    }
}
