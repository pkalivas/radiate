use crate::chart::RollingLineChart;
use radiate_engines::{Metric, stats::TagType};
use radiate_utils::SmallStr;
use std::collections::HashMap;

const MAX_CHART_POINTS: usize = 1000;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetricChartType {
    Last,
    Mean,
    Stddev,
    Variance,
    BoxWhisker,
    Distribution,
}

impl MetricChartType {
    // Scalar/Time metrics: their per-generation value and its running average.
    // Stddev/Variance over generations are rarely what you want for a scalar, so
    // they're kept out of the scalar set to reduce noise.
    pub(crate) const SCALAR_VIEWS: &'static [MetricChartType] =
        &[MetricChartType::Last, MetricChartType::Mean];
    // Distribution metrics: the population's center and spread over generations.
    // `Last` is meaningless (no single value), so it's excluded; the within-gen
    // Histogram and cross-gen quantile Bands views slot in here next.
    pub(crate) const DISTRIBUTION_VIEWS: &'static [MetricChartType] = &[
        MetricChartType::BoxWhisker,
        MetricChartType::Distribution,
        MetricChartType::Mean,
        MetricChartType::Stddev,
        MetricChartType::Variance,
    ];

    /// The ordered set of chart views a metric supports, driven by its tags —
    /// the chart panel asks the metric what it can show, mirroring how the
    /// detail panel already branches on tag.
    pub fn for_metric(metric: &Metric) -> &'static [MetricChartType] {
        if metric.contains_tag(&TagType::Distribution) {
            Self::DISTRIBUTION_VIEWS
        } else {
            // Statistic and Time both read as value + running mean.
            Self::SCALAR_VIEWS
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MetricChartType::Last => "Last",
            MetricChartType::Mean => "Mean",
            MetricChartType::Stddev => "Stddev",
            MetricChartType::Variance => "Variance",
            MetricChartType::BoxWhisker => "Box & Whisker",
            MetricChartType::Distribution => "Distribution",
        }
    }
}

pub struct ChartState {
    value_charts: HashMap<SmallStr, RollingLineChart>,
    mean_charts: HashMap<SmallStr, RollingLineChart>,
    stddev_charts: HashMap<SmallStr, RollingLineChart>,
    variance_charts: HashMap<SmallStr, RollingLineChart>,
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
        key: &str,
        chart_type: MetricChartType,
    ) -> Option<&RollingLineChart> {
        match chart_type {
            MetricChartType::Last => self.value_charts.get(key),
            MetricChartType::Mean => self.mean_charts.get(key),
            MetricChartType::Stddev => self.stddev_charts.get(key),
            MetricChartType::Variance => self.variance_charts.get(key),
            _ => None,
        }
    }

    pub fn update_from_metric(&mut self, metric: &Metric) {
        let stat = metric.statistic();
        let key = metric.name();

        if !metric.contains_tag(&TagType::Distribution) {
            let value_chart = self.get_or_create_chart(key, MetricChartType::Last);
            value_chart.push(stat.last_value() as f64);
        }

        let mean_chart = self.get_or_create_chart(key, MetricChartType::Mean);
        mean_chart.push(stat.mean() as f64);

        let stddev_chart = self.get_or_create_chart(key, MetricChartType::Stddev);
        stddev_chart.push(stat.std_dev().unwrap_or(0.0) as f64);

        let variance_chart = self.get_or_create_chart(key, MetricChartType::Variance);
        variance_chart.push(stat.variance().unwrap_or(0.0) as f64);
    }

    fn get_or_create_chart(
        &mut self,
        key: &SmallStr,
        chart_type: MetricChartType,
    ) -> &mut RollingLineChart {
        match chart_type {
            MetricChartType::Last => self.value_charts.entry(key.clone()).or_insert_with(|| {
                RollingLineChart::with_capacity(MAX_CHART_POINTS)
                    .with_title(key.as_str())
                    .with_color(ratatui::style::Color::LightCyan)
            }),
            MetricChartType::Mean => self.mean_charts.entry(key.clone()).or_insert_with(|| {
                RollingLineChart::with_capacity(MAX_CHART_POINTS)
                    .with_title(format!("{} μ (mean)", key))
                    .with_color(ratatui::style::Color::Yellow)
            }),
            MetricChartType::Stddev => self.stddev_charts.entry(key.clone()).or_insert_with(|| {
                RollingLineChart::with_capacity(MAX_CHART_POINTS)
                    .with_title(format!("{} σ (stddev)", key))
                    .with_color(ratatui::style::Color::LightGreen)
            }),
            MetricChartType::Variance => {
                self.variance_charts.entry(key.clone()).or_insert_with(|| {
                    RollingLineChart::with_capacity(MAX_CHART_POINTS)
                        .with_title(format!("{} σ² (variance)", key))
                        .with_color(ratatui::style::Color::LightBlue)
                })
            }
            _ => panic!("Unsupported chart type for line chart"),
        }
    }
}
