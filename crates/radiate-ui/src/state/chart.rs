use crate::chart::RollingChart;
use radiate_engines::{Metric, stats::TagType};
use radiate_utils::intern;
use std::collections::HashMap;

const MAX_CHART_POINTS: usize = 1000;

const CHART_NAMES: &[&'static str] = &["Value", "Mean", "Stddev", "Variance"];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChartType {
    Value,
    Mean,
    Stddev,
    Variance,
}

impl ChartType {
    pub fn chart_options() -> &'static [&'static str] {
        CHART_NAMES
    }

    pub fn next(self) -> Self {
        match self {
            ChartType::Value => ChartType::Mean,
            ChartType::Mean => ChartType::Stddev,
            ChartType::Stddev => ChartType::Variance,
            ChartType::Variance => ChartType::Value,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            ChartType::Value => ChartType::Variance,
            ChartType::Mean => ChartType::Value,
            ChartType::Stddev => ChartType::Mean,
            ChartType::Variance => ChartType::Stddev,
        }
    }
}

pub struct ChartState {
    value_charts: HashMap<&'static str, RollingChart>,
    mean_charts: HashMap<&'static str, RollingChart>,
    stddev_charts: HashMap<&'static str, RollingChart>,
    variance_charts: HashMap<&'static str, RollingChart>,
}

impl ChartState {
    pub fn new() -> Self {
        Self {
            value_charts: HashMap::new(),
            mean_charts: HashMap::new(),
            stddev_charts: HashMap::new(),
            variance_charts: HashMap::new(),
        }
    }

    pub fn get_by_key(&self, key: &'static str, chart_type: ChartType) -> Option<&RollingChart> {
        match chart_type {
            ChartType::Value => self.value_charts.get(key),
            ChartType::Mean => self.mean_charts.get(key),
            ChartType::Stddev => self.stddev_charts.get(key),
            ChartType::Variance => self.variance_charts.get(key),
        }
    }

    pub fn update_from_metric(&mut self, metric: &Metric) {
        let stat = metric.statistic();
        let key = intern!(metric.name());
        if !metric.contains_tag(&TagType::Distribution) {
            let value_chart = self.get_or_create_chart(key, ChartType::Value);
            value_chart.push(stat.last_value() as f64);
        }

        let mean_chart = self.get_or_create_chart(key, ChartType::Mean);
        mean_chart.push(stat.mean() as f64);

        let stddev_chart = self.get_or_create_chart(key, ChartType::Stddev);
        stddev_chart.push(stat.std_dev().unwrap_or(0.0) as f64);

        let variance_chart = self.get_or_create_chart(key, ChartType::Variance);
        variance_chart.push(stat.variance().unwrap_or(0.0) as f64);
    }

    fn get_or_create_chart(
        &mut self,
        key: &'static str,
        chart_type: ChartType,
    ) -> &mut RollingChart {
        match chart_type {
            ChartType::Value => self.value_charts.entry(key).or_insert_with(|| {
                RollingChart::with_capacity(MAX_CHART_POINTS)
                    .with_title(key)
                    .with_color(ratatui::style::Color::LightCyan)
            }),
            ChartType::Mean => self.mean_charts.entry(key).or_insert_with(|| {
                RollingChart::with_capacity(MAX_CHART_POINTS)
                    .with_title(format!("{} μ (mean)", key))
                    .with_color(ratatui::style::Color::Yellow)
            }),
            ChartType::Stddev => self.stddev_charts.entry(key).or_insert_with(|| {
                RollingChart::with_capacity(MAX_CHART_POINTS)
                    .with_title(format!("{} σ (stddev)", key))
                    .with_color(ratatui::style::Color::LightGreen)
            }),
            ChartType::Variance => self.variance_charts.entry(key).or_insert_with(|| {
                RollingChart::with_capacity(MAX_CHART_POINTS)
                    .with_title(format!("{} σ² (variance)", key))
                    .with_color(ratatui::style::Color::LightBlue)
            }),
        }
    }
}
