use crate::chart::RollingLineChart;
use radiate_engines::{Metric, stats::TagType};
use radiate_utils::intern;
use std::collections::HashMap;

const MAX_CHART_POINTS: usize = 1000;
const CHART_NAMES: &[&str] = &["Value", "Mean", "Stddev", "Variance"];

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineChartType {
    Value,
    Mean,
    Stddev,
    Variance,
}

impl LineChartType {
    pub fn chart_options() -> &'static [&'static str] {
        CHART_NAMES
    }

    pub fn next(self) -> Self {
        match self {
            LineChartType::Value => LineChartType::Mean,
            LineChartType::Mean => LineChartType::Stddev,
            LineChartType::Stddev => LineChartType::Variance,
            LineChartType::Variance => LineChartType::Value,
        }
    }

    pub fn previous(self) -> Self {
        match self {
            LineChartType::Value => LineChartType::Variance,
            LineChartType::Mean => LineChartType::Value,
            LineChartType::Stddev => LineChartType::Mean,
            LineChartType::Variance => LineChartType::Stddev,
        }
    }
}

pub struct ChartState {
    value_charts: HashMap<&'static str, RollingLineChart>,
    mean_charts: HashMap<&'static str, RollingLineChart>,
    stddev_charts: HashMap<&'static str, RollingLineChart>,
    variance_charts: HashMap<&'static str, RollingLineChart>,
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

    pub fn get_line_chart(
        &self,
        key: &'static str,
        chart_type: LineChartType,
    ) -> Option<&RollingLineChart> {
        match chart_type {
            LineChartType::Value => self.value_charts.get(key),
            LineChartType::Mean => self.mean_charts.get(key),
            LineChartType::Stddev => self.stddev_charts.get(key),
            LineChartType::Variance => self.variance_charts.get(key),
        }
    }

    pub fn update_from_metric(&mut self, metric: &Metric) {
        let stat = metric.statistic();
        let key = intern!(metric.name());
        if !metric.contains_tag(&TagType::Distribution) {
            let value_chart = self.get_or_create_chart(key, LineChartType::Value);
            value_chart.push(stat.last_value() as f64);
        }

        let mean_chart = self.get_or_create_chart(key, LineChartType::Mean);
        mean_chart.push(stat.mean() as f64);

        let stddev_chart = self.get_or_create_chart(key, LineChartType::Stddev);
        stddev_chart.push(stat.std_dev().unwrap_or(0.0) as f64);

        let variance_chart = self.get_or_create_chart(key, LineChartType::Variance);
        variance_chart.push(stat.variance().unwrap_or(0.0) as f64);
    }

    fn get_or_create_chart(
        &mut self,
        key: &'static str,
        chart_type: LineChartType,
    ) -> &mut RollingLineChart {
        match chart_type {
            LineChartType::Value => self.value_charts.entry(key).or_insert_with(|| {
                RollingLineChart::with_capacity(MAX_CHART_POINTS)
                    .with_title(key)
                    .with_color(ratatui::style::Color::LightCyan)
            }),
            LineChartType::Mean => self.mean_charts.entry(key).or_insert_with(|| {
                RollingLineChart::with_capacity(MAX_CHART_POINTS)
                    .with_title(format!("{} μ (mean)", key))
                    .with_color(ratatui::style::Color::Yellow)
            }),
            LineChartType::Stddev => self.stddev_charts.entry(key).or_insert_with(|| {
                RollingLineChart::with_capacity(MAX_CHART_POINTS)
                    .with_title(format!("{} σ (stddev)", key))
                    .with_color(ratatui::style::Color::LightGreen)
            }),
            LineChartType::Variance => self.variance_charts.entry(key).or_insert_with(|| {
                RollingLineChart::with_capacity(MAX_CHART_POINTS)
                    .with_title(format!("{} σ² (variance)", key))
                    .with_color(ratatui::style::Color::LightBlue)
            }),
        }
    }
}
