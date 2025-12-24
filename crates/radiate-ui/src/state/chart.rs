use crate::chart::RollingChart;
use radiate_engines::{Metric, stats::TagKind};
use radiate_utils::intern;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChartType {
    Value,
    Mean,
}

pub struct ChartState {
    fitness_chart: RollingChart,
    fitness_mean_chart: RollingChart,
    value_charts: HashMap<&'static str, RollingChart>,
    mean_charts: HashMap<&'static str, RollingChart>,
}

impl ChartState {
    pub fn new() -> Self {
        Self {
            fitness_chart: RollingChart::with_capacity(1000)
                .with_title("Score")
                .with_color(ratatui::style::Color::LightCyan),
            fitness_mean_chart: RollingChart::with_capacity(1000)
                .with_title("μ (mean)")
                .with_color(ratatui::style::Color::Yellow),
            value_charts: HashMap::new(),
            mean_charts: HashMap::new(),
        }
    }

    pub fn fitness_chart(&self) -> &RollingChart {
        &self.fitness_chart
    }

    pub fn fitness_chart_mut(&mut self) -> &mut RollingChart {
        &mut self.fitness_chart
    }

    pub fn fitness_mean_chart(&self) -> &RollingChart {
        &self.fitness_mean_chart
    }

    pub fn fitness_mean_chart_mut(&mut self) -> &mut RollingChart {
        &mut self.fitness_mean_chart
    }

    pub fn get_by_key(&self, key: &'static str, chart_type: ChartType) -> Option<&RollingChart> {
        match chart_type {
            ChartType::Value => self.value_charts.get(key),
            ChartType::Mean => self.mean_charts.get(key),
        }
    }

    pub fn get_or_create_chart(
        &mut self,
        key: &'static str,
        chart_type: ChartType,
    ) -> &mut RollingChart {
        match chart_type {
            ChartType::Value => self.value_charts.entry(key).or_insert_with(|| {
                RollingChart::with_capacity(1000)
                    .with_title(key)
                    .with_color(ratatui::style::Color::LightCyan)
            }),
            ChartType::Mean => self.mean_charts.entry(key).or_insert_with(|| {
                RollingChart::with_capacity(1000)
                    .with_title("μ (mean)")
                    .with_color(ratatui::style::Color::Yellow)
            }),
        }
    }

    pub fn update_from_metric(&mut self, metric: &Metric) {
        if let Some(stat) = metric.statistic() {
            let key = intern!(metric.name());
            if !metric.contains_tag(&TagKind::Distribution) {
                let value_chart = self.get_or_create_chart(key, ChartType::Value);
                value_chart.add_value((value_chart.len() as f64, stat.last_value() as f64));
            }

            let mean_chart = self.get_or_create_chart(key, ChartType::Mean);
            mean_chart.add_value((mean_chart.len() as f64, stat.mean() as f64));
        }
    }
}
