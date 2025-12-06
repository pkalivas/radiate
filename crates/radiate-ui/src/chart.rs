use radiate_engines::Metric;
use ratatui::{
    layout::Constraint,
    style::{Color, Style, Stylize},
    symbols,
    text::Line,
    widgets::{Axis, Block, Chart, Dataset, GraphType},
};

pub struct ChartInner {
    title: String,
    min_y: f64,
    max_y: f64,
    values: Vec<(f64, f64)>,
    color: Color,
    capacity: usize,
}

impl ChartInner {
    /// Explicit capacity constructor
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            title: "".to_string(),
            min_y: f64::MAX,
            max_y: f64::MIN,
            values: Vec::with_capacity(capacity),
            color: Color::White,
            capacity,
        }
    }

    pub fn with_title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn values(&self) -> &[(f64, f64)] {
        &self.values
    }

    pub fn add_value(&mut self, value: (f64, f64)) {
        self.values.push(value);

        // If we exceed capacity, drop oldest and recompute bounds
        if self.values.len() > self.capacity {
            let overflow = self.values.len() - self.capacity;
            self.values.drain(0..overflow);
            self.recompute_bounds();
        } else {
            // Fast incremental update
            let y = value.1;
            if y < self.min_y {
                self.min_y = y;
            }
            if y > self.max_y {
                self.max_y = y;
            }
        }
    }

    /// Replace values with a new sequence.
    /// Keeps only the last `capacity` entries if there are more.
    pub fn set_values(&mut self, values: &[f32]) {
        self.values.clear();

        // keep only the tail up to capacity
        let keep = values.len().min(self.capacity);
        let start = values.len().saturating_sub(keep);
        self.min_y = f64::MAX;
        self.max_y = f64::MIN;

        for (i, val) in values.iter().enumerate().skip(start) {
            let f_val = *val as f64;

            if f_val < self.min_y {
                self.min_y = f_val;
            }
            if f_val > self.max_y {
                self.max_y = f_val;
            }

            self.values.push((i as f64, f_val));
        }
    }

    fn recompute_bounds(&mut self) {
        self.min_y = f64::MAX;
        self.max_y = f64::MIN;

        for &(_, y) in &self.values {
            if y < self.min_y {
                self.min_y = y;
            }
            if y > self.max_y {
                self.max_y = y;
            }
        }

        if self.values.is_empty() {
            self.min_y = f64::MAX;
            self.max_y = f64::MIN;
        }
    }
}

pub struct ChartData {
    name: String,
    metric_name: &'static str,
    last_value: ChartInner,
    mean_value: Option<ChartInner>,
}

impl ChartData {
    /// Default constructor with a reasonable capacity (e.g., 1000 points)
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    /// Explicit capacity constructor
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            name: "".to_string(),
            metric_name: "",
            last_value: ChartInner::with_capacity(capacity).with_color(Color::Cyan),
            mean_value: None,
        }
    }

    pub fn with_metric_name(mut self, metric_name: &'static str) -> Self {
        self.metric_name = metric_name;
        self.last_value.title = metric_name.into();
        self
    }

    pub fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.name = name.into();
        self.last_value.title = self.name.clone();
        self
    }

    pub fn update_last_value(&mut self, index: f64, value: f64) {
        self.last_value.add_value((index, value));
    }

    pub fn update_mean_value(&mut self, index: f64, value: f64) {
        if let Some(mean_chart) = &mut self.mean_value {
            mean_chart.add_value((index, value));
        } else {
            let mut new_inner =
                ChartInner::with_capacity(self.last_value.capacity).with_title("μ (mean)");
            new_inner.add_value((index, value));
            self.mean_value = Some(new_inner);
        }
    }

    pub fn update(&mut self, metric: &Metric) {
        if let Some(stat) = metric.statistic() {
            let len = self.last_value.values.len() as f64;
            self.last_value.add_value((len, stat.last_value() as f64));

            if let Some(mean_chart) = &mut self.mean_value {
                let mean_len = mean_chart.values.len() as f64;
                mean_chart.add_value((mean_len, stat.mean() as f64));
            } else {
                let mut new_inner =
                    ChartInner::with_capacity(self.last_value.capacity).with_title("μ (mean)");
                new_inner.add_value((0.0, stat.mean() as f64));
                self.mean_value = Some(new_inner);
            }
        } else if let Some(dist) = metric.distribution() {
            self.last_value.set_values(&dist.last_sequence);

            if let Some(mean_chart) = &mut self.mean_value {
                let mean_len = mean_chart.values.len() as f64;
                mean_chart.add_value((mean_len, dist.mean() as f64));
            } else {
                let mut new_inner =
                    ChartInner::with_capacity(self.last_value.capacity).with_title("μ (mean)");
                new_inner.add_value((0.0, dist.mean() as f64));
                self.mean_value = Some(new_inner);
            }
        }
    }

    pub fn min_y(&self) -> f64 {
        if let Some(mean) = &self.mean_value {
            self.last_value.min_y.min(mean.min_y)
        } else {
            self.last_value.min_y
        }
    }

    pub fn max_y(&self) -> f64 {
        if let Some(mean) = &self.mean_value {
            self.last_value.max_y.max(mean.max_y)
        } else {
            self.last_value.max_y
        }
    }

    pub fn min_x(&self) -> f64 {
        self.last_value
            .values
            .first()
            .map(|(x, _)| *x)
            .unwrap_or(0.0)
    }

    pub fn max_x(&self) -> f64 {
        self.last_value
            .values
            .last()
            .map(|(x, _)| *x)
            .unwrap_or(0.0)
    }

    pub fn x_bounds(&self) -> (f64, f64) {
        (self.min_x(), self.max_x())
    }

    pub fn y_bounds(&self) -> (f64, f64) {
        (self.min_y(), self.max_y())
    }

    pub fn create_chart_widgets<'a>(&'a self) -> ratatui::widgets::Chart<'a> {
        if let Some(mean) = &self.mean_value {
            self.chart_widget(vec![&self.last_value, mean])
        } else {
            self.chart_widget(vec![&self.last_value])
        }
    }

    pub fn create_value_chart_widget<'a>(&'a self) -> ratatui::widgets::Chart<'a> {
        self.chart_widget(vec![&self.last_value])
    }

    #[allow(dead_code)]
    pub fn create_mean_chart_widget<'a>(&'a self) -> Option<ratatui::widgets::Chart<'a>> {
        if let Some(mean) = &self.mean_value {
            Some(self.chart_widget(vec![mean]))
        } else {
            None
        }
    }

    fn chart_widget<'a>(&'a self, charts: Vec<&'a ChartInner>) -> ratatui::widgets::Chart<'a> {
        let (min_x, max_x) = self.x_bounds();
        let (min_y, max_y) = self.y_bounds();

        let mid_y = (min_y + max_y) / 2.0;

        let datasets = charts
            .iter()
            .map(|dim| {
                Dataset::default()
                    .name(dim.title())
                    .marker(symbols::Marker::Braille)
                    .style(Style::default().fg(dim.color()))
                    .graph_type(GraphType::Line)
                    .data(dim.values())
            })
            .collect::<Vec<_>>();

        Chart::new(datasets)
            .block(Block::bordered())
            .x_axis(
                Axis::default()
                    .style(Style::default().gray())
                    .bounds([min_x, max_x]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().gray())
                    .bounds([min_y, max_y])
                    .labels(Line::from(vec![
                        format!("{:.2}", min_y).bold().into(),
                        format!("{:.2}", mid_y).into(),
                        format!("{:.2}", max_y).bold().into(),
                    ])),
            )
            .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)))
    }
}

impl Default for ChartData {
    fn default() -> Self {
        Self::with_capacity(1000).with_name("Score")
    }
}
